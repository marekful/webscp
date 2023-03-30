<template>
  <div class="card floating">
    <div class="card-title">
      <h2>{{ $t("prompts.copy") }}</h2>
    </div>

    <div class="card-content">
      <server-select
        @update:selected="(val) => changeServer(val)"
      ></server-select>

      <file-list
        @update:selected="(val) => (dest = val)"
        :agent-id="agentId"
      ></file-list>
    </div>

    <div class="card-action">
      <button
        class="button button--flat button--grey"
        @click="$store.commit('closeHovers')"
        :aria-label="$t('buttons.cancel')"
        :title="$t('buttons.cancel')"
      >
        {{ $t("buttons.cancel") }}
      </button>
      <button
        class="button button--flat"
        @click="copy"
        :aria-label="$t('buttons.copy')"
        :title="$t('buttons.copy')"
      >
        {{ $t("buttons.copy") }}
      </button>
    </div>
  </div>
</template>

<script>
import { mapState } from "vuex";
import FileList from "./FileList";
import ServerSelect from "../ServerSelect";
import { files as api } from "@/api";
import { remote_files as remote_api } from "@/api";
import buttons from "@/utils/buttons";
import * as upload from "@/utils/upload";

export default {
  name: "copy",
  components: { FileList, ServerSelect },
  data: function () {
    return {
      current: window.location.pathname,
      dest: null,
      agentId: null,
      sseClient: null,
    };
  },
  computed: mapState(["req", "selected", "transfers"]),
  methods: {
    changeServer: function (val) {
      if (val === 0) {
        this.agentId = 0;
      } else {
        this.agentId = val;
      }
    },
    copy: function (event) {
      event.preventDefault();

      // Create a new promise for each file.
      let items = [];
      for (let item of this.selected) {
        items.push({
          from: this.req.items[item].url,
          to: this.dest + encodeURIComponent(this.req.items[item].name),
          name: this.req.items[item].name,
        });
      }

      if (this.agentId === 0) {
        return this.localCopy(items);
      } else {
        return this.remoteCopy(this.agentId, items);
      }
    },
    localCopy: async function (items) {
      let action = async (overwrite, rename) => {
        buttons.loading("copy");

        await api
          .copy(items, overwrite, rename)
          .then(() => {
            buttons.success("copy");

            if (this.$route.path === this.dest) {
              this.$store.commit("setReload", true);

              return;
            }

            this.$router.push({ path: this.dest });
          })
          .catch((e) => {
            buttons.done("copy");
            this.$showError(e);
          });
      };

      if (this.$route.path === this.dest) {
        this.$store.commit("closeHovers");
        action(false, true);

        return;
      }

      let dstItems = (await api.fetch(this.dest)).items;
      let conflict = upload.checkConflict(items, dstItems);

      let overwrite = false;
      let rename = false;

      if (conflict) {
        this.$store.commit("showHover", {
          prompt: "replace-rename",
          confirm: (event, option) => {
            overwrite = option == "overwrite";
            rename = option == "rename";

            event.preventDefault();
            this.$store.commit("closeHovers");
            action(overwrite, rename);
          },
        });

        return;
      }

      action(overwrite, rename);
    },
    remoteCopy: async function (agentId, items) {
      let action = async (overwrite, rename) => {
        await remote_api
          .copyStart(agentId, items, overwrite, rename)
          .then((res) => {
            let transferId = res.message;
            this.$store.commit("addTransfer", {id: transferId});
            this.sseClient = new EventSource(
              `/api/sse/transfer/${transferId}/poll`
            );
            this.sseClient.onmessage = this.sseMessage;
            this.sseClient.transferId = transferId;
            this.$store.commit("closeHovers");

            setTimeout(() => {
              buttons.active("transfers");
              buttons.loading("transfers");
            }, 100);

          })
          .catch((e) => {
            buttons.done("copy");
            this.$showError(e);
          });
      };

      let overwrite = false;
      let rename = false;

      action(overwrite, rename);
    },
    sseMessage(event) {
      if (!event.isTrusted) return;

      switch (event.data) {
        case "archiving":
          break;
        case "uploading":
          break;
        case "extracting":
          break;
        case "complete":
          buttons.successPromise("transfers")
              .then(() => buttons.active("transfers", false))
              .catch(() => buttons.active("transfers", false));vv
          this.sseClient.close();
          this.$store.commit("removeTransfer", this.sseClient.transferId);
          this.sseClient = null;
          break;
        default:
          this.$showError(event.data);
          buttons.done("transfers");
          buttons.active("transfers", false);
          this.sseClient.close();
          this.sseClient = null;
      }
    },
  },
};
</script>
