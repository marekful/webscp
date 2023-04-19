<template>
  <div class="top">
    <div v-if="remoteLoading" class="remote-loading">
      <h2 class="message delayed">
        <div class="spinner">
          <div class="bounce1"></div>
          <div class="bounce2"></div>
          <div class="bounce3"></div>
        </div>
        <span>{{ $t("files.loading") }}</span>
      </h2>
    </div>
    <p>
      <code>{{ nav }}</code>
    </p>
    <ul class="file-list">
      <li
        @click="itemClick"
        @touchstart="touchstart"
        @dblclick="next"
        role="button"
        tabindex="0"
        :aria-label="item.name"
        :aria-selected="selected === item.url"
        :key="item.name"
        v-for="item in items"
        :data-url="item.url"
      >
        {{ item.name }}
      </li>
    </ul>
  </div>
</template>

<script>
import { mapState } from "vuex";
import url from "@/utils/url";
import { files, remote_files } from "@/api";

export default {
  name: "file-list",
  data: function () {
    return {
      items: [],
      touches: {
        id: "",
        count: 0,
      },
      selected: null,
      current: window.location.pathname,
      remoteLoading: false,
    };
  },
  props: {
    agentId: {
      type: Number,
      default: 0,
    },
  },
  watch: {
    agentId(agent_id) {
      if (agent_id === 0) {
        this.fillOptions(this.req);
      } else {
        this.remote(agent_id, "/files/");
      }
    },
  },
  computed: {
    ...mapState(["req", "user"]),
    nav() {
      return decodeURIComponent(this.current);
    },
  },
  methods: {
    fillOptions(req) {
      // Sets the current path and resets
      // the current items.
      this.current = req.url;
      this.items = [];

      this.$emit("update:selected", this.current);

      // If the path isn't the root path,
      // show a button to navigate to the previous
      // directory.
      if (req.url !== "/files/") {
        this.items.push({
          name: "..",
          url: url.removeLastDir(req.url) + "/",
        });
      }

      // If this folder is empty, finish here.
      if (req.items === null) return;

      // Otherwise we add every directory to the
      // move options.
      for (let item of req.items) {
        if (!item.isDir) continue;

        this.items.push({
          name: item.name,
          url: item.url,
        });
      }
    },
    next: function (event) {
      // Retrieves the URL of the directory the user
      // just clicked in and fill the options with its
      // content.
      let uri = event.currentTarget.dataset.url;

      if (this.agentId === 0) {
        files.fetch(uri).then(this.fillOptions).catch(this.$showError);
      } else {
        this.remoteLoading = true;
        remote_files
          .fetch(this.agentId, uri)
          .then((res) => {
            this.resetItems();
            this.fillOptions(res);
          })
          .catch(this.remoteError)
          .finally(() => (this.remoteLoading = false));
      }
    },
    remote: function (agent_id, uri) {
      this.remoteLoading = true;
      remote_files
        .fetch(agent_id, uri)
        .then((res) => {
          this.resetItems();
          this.fillOptions(res);
        })
        .catch(this.remoteError)
        .finally(() => (this.remoteLoading = false));
    },
    remoteError(error) {
      if (error.status === 511) {
        this.$store.commit("setLoginAgent", {
          id: this.agentId,
          component: typeof this.$parent.copy === "function" ? "copy" : "move",
        });
        this.$store.commit("showHover", "agent-login");
        return;
      }
      this.$showError(error);
    },
    resetItems() {
      this.items = [];
      this.current = "";
    },
    touchstart(event) {
      let url = event.currentTarget.dataset.url;

      // In 300 milliseconds, we shall reset the count.
      setTimeout(() => {
        this.touches.count = 0;
      }, 300);

      // If the element the user is touching
      // is different from the last one he touched,
      // reset the count.
      if (this.touches.id !== url) {
        this.touches.id = url;
        this.touches.count = 1;
        return;
      }

      this.touches.count++;

      // If there is more than one touch already,
      // open the next screen.
      if (this.touches.count > 1) {
        this.next(event);
      }
    },
    itemClick: function (event) {
      if (this.user.singleClick) this.next(event);
      else this.select(event);
    },
    select: function (event) {
      // If the element is already selected, unselect it.
      if (this.selected === event.currentTarget.dataset.url) {
        this.selected = null;
        this.$emit("update:selected", this.current);
        return;
      }

      // Otherwise select the element.
      this.selected = event.currentTarget.dataset.url;
      this.$emit("update:selected", this.selected);
    },
  },
};
</script>

<style>
.top {
  position: relative;
}
.remote-loading {
  position: absolute;
  z-index: 9999;
  background: rgba(255, 255, 255, 0.33);
  width: 100%;
  height: 100%;
  top: 0;
  left: 0;
}

.remote-loading .message {
  height: 100%;
}
</style>
