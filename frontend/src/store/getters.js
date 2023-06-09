const getters = {
  isLogged: (state) => state.user !== null,
  isFiles: (state) => !state.loading && state.route.name === "Files",
  isListing: (state, getters) => getters.isFiles && state.req.isDir,
  selectedCount: (state) => state.selected.length,
  transfersInProgress: (state) => {
    let inProgress = 0;
    for (let tr of state.transfers) {
      if (state.user.id !== tr.agent.user.id) continue;
      if (tr.pending) {
        inProgress += 1;
      }
    }
    return inProgress;
  },
  transfersTotal: (state) => {
    let total = 0;
    for (let tr of state.transfers) {
      if (state.user.id !== tr.agent.user.id) continue;
      total += 1;
    }
    return total;
  },
  progress: (state) => {
    if (state.upload.progress.length === 0) {
      return 0;
    }

    let totalSize = state.upload.sizes.reduce((a, b) => a + b, 0);

    let sum = state.upload.progress.reduce((acc, val) => acc + val);
    return Math.ceil((sum / totalSize) * 100);
  },
  filesInUploadCount: (state) => {
    let total =
      Object.keys(state.upload.uploads).length + state.upload.queue.length;
    return total;
  },
  filesInUpload: (state) => {
    let files = [];

    for (let index in state.upload.uploads) {
      let upload = state.upload.uploads[index];
      let id = upload.id;
      let type = upload.type;
      let name = upload.file.name;
      let size = state.upload.sizes[id];
      let isDir = upload.file.isDir;
      let progress = isDir
        ? 100
        : Math.ceil((state.upload.progress[id] / size) * 100);

      files.push({
        id,
        name,
        progress,
        type,
        isDir,
      });
    }

    return files.sort((a, b) => a.progress - b.progress);
  },
};

export default getters;
