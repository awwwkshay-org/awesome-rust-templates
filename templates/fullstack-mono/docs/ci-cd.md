# CI and delivery

The workflows deliberately separate verification from delivery:

- `.github/workflows/ci.yml` validates every pull request, merge queue entry,
  and push to `main`.
- `.github/workflows/cd.yml` publishes images from a GitHub release or a manual
  run. It can optionally propose a production deployment in a separate GitOps
  repository.
- `.github/dependabot.yml` keeps Rust dependencies, GitHub Actions, and Docker
  base images current.

## Continuous integration

CI runs formatting, Clippy, and tests with PostgreSQL. It checks the Dioxus SSR
server and WebAssembly client independently, then builds both production
Dockerfiles. Cargo commands use the committed lockfile and warnings fail the
build.

Protect `main` in GitHub and require these checks before merging:

- `Rust checks`
- `Build API image`
- `Build UI SSR image`

Also require pull-request approval, dismiss stale approvals, block force pushes,
require conversation resolution, and enable the merge queue if the repository
receives concurrent changes.

## Container publishing

A published GitHub release builds and pushes:

- `ghcr.io/<owner>/<repository>-api`
- `ghcr.io/<owner>/<repository>-ui`

Each image receives an immutable `sha-<commit>` tag. Releases also receive the
release's semantic-version tag, its `major.minor` tag, and `latest` for a
non-prerelease. The workflow publishes BuildKit provenance, an SBOM, and a
GitHub artifact attestation. Deployment output always uses the image digest,
not a mutable tag.

The default platform is `linux/amd64`. Set the repository variable
`IMAGE_PLATFORMS` to a comma-separated value such as
`linux/amd64,linux/arm64` when production needs a multi-architecture image.

GitHub's `GITHUB_TOKEN` publishes packages without a separate registry secret.
After the first publish, confirm the packages are linked to the repository and
set their visibility and pull permissions for the target runtime.

## Optional GitOps deployment proposal

Delivery does not mutate a cluster directly. Following the pattern used by the
reference repositories, it updates a Kustomize overlay with immutable digests,
validates the rendered manifests, and opens a pull request. The job is skipped
unless both of these repository variables exist:

| Setting | Required | Meaning |
| --- | --- | --- |
| `INFRA_REPOSITORY` | yes | Infrastructure repository as `owner/name` |
| `INFRA_KUSTOMIZE_DIR` | yes | Overlay directory within that repository |
| `INFRA_BASE_BRANCH` | no | Target branch; defaults to `main` |
| `API_KUSTOMIZE_NAME` | no | API image name in `kustomization.yaml`; defaults to `api` |
| `UI_KUSTOMIZE_NAME` | no | UI image name in `kustomization.yaml`; defaults to `ui` |

Add `INFRA_REPO_TOKEN` as a secret in the `production` GitHub environment. Use
a fine-grained token or GitHub App token limited to contents and pull requests
for the one infrastructure repository. The Kustomization must already contain
image entries matching `API_KUSTOMIZE_NAME` and `UI_KUSTOMIZE_NAME`.

Configure the `production` environment with required reviewers, prevent
self-review, and limit deployment to protected tags/branches. On a release, the
environment gate controls the proposal. On a manual run, select
`propose_deployment` to request it.

If no infrastructure repository is configured, delivery remains fully useful:
it publishes verifiable images and skips only the deployment proposal.

## Release procedure

1. Merge a change only after all required CI checks pass.
2. Create a semantic-version GitHub release from the intended commit.
3. Approve the `production` environment when the image-publishing job finishes.
4. Review and merge the generated infrastructure pull request.
5. Let the deployment platform reconcile the approved digest change and verify
   its health checks and rollback policy.

For the strongest supply-chain policy, pin third-party GitHub Actions to full
commit SHAs at repository adoption time and let Dependabot update those pins.
The template uses readable major-version tags so new repositories receive the
maintainers' compatible security fixes immediately.
