# renovate: datasource=node depName=node
ARG NODE_VERSION=18.14.2

FROM node:${NODE_VERSION}-alpine

# renovate: datasource=npm depName=pnpm
ARG PNPM_VERSION=7.28.0
ARG GIT_REPO_CRATESIO_URL=https://github.com/rust-lang/crates.io
ARG GIT_REPO_CRATESIO_COMMIT_ID=4f77e83728164c2c96f9ad79d5cd625484b1e995

WORKDIR /app

# Install `pnpm`
RUN npm install --global pnpm@$PNPM_VERSION

# Install git
RUN apk update && apk add git

RUN git clone "${GIT_REPO_CRATESIO_URL}" crates_io \
    && cd crates_io \
    && git checkout "${GIT_REPO_CRATESIO_COMMIT_ID}" \
    && cd -
RUN cp ./crates_io/pnpm-lock.yaml /app/pnpm-lock.yaml && pnpm fetch
RUN cp -r /app/crates_io/* /app && rm -r ./crates_io

# Install dependencies from previously downloaded pnpm store
RUN pnpm install --offline
ENTRYPOINT ["pnpm", "start:staging"]
