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
      <span v-if="showOptions" class="options">
        <input
          type="checkbox"
          name="compress"
          id="compress"
          ref="compress"
          :checked="compress"
          @click="setCompress"
        />
        <label for="compress">{{ $t("prompts.agent.compress") }}</label>
      </span>
      <button
        class="button button--flat button--grey"
        ref="cancel"
        @click="$store.commit('closeHovers')"
        :aria-label="$t('buttons.cancel')"
        :title="$t('buttons.cancel')"
      >
        {{ $t("buttons.cancel") }}
      </button>
      <button
        class="button button--flat"
        ref="submit"
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
import transfers from "@/utils/transfers";
import * as upload from "@/utils/upload";

export default {
  name: "copy",
  components: { FileList, ServerSelect },
  data: function () {
    return {
      current: window.location.pathname,
      dest: null,
      agentId: null,
      agent: null,
      compress: false,
      showOptions: false,
    };
  },
  computed: mapState(["req", "selected", "transfers"]),
  methods: {
    changeServer: function (val) {
      let id = val.id;
      if (id === 0) {
        this.agentId = 0;
        this.showOptions = false;
      } else {
        this.agentId = id;
        this.agent = val;
        this.showOptions = true;
      }
    },
    setCompress() {
      this.compress = !this.compress;
    },
    disableControls() {
      this.$refs.submit.setAttribute("disabled", "disabled");
      this.$refs.cancel.setAttribute("disabled", "disabled");
      this.$refs.compress.setAttribute("disabled", "disabled");
    },
    enableControls() {
      if (!this.$refs.submit) return;
      this.$refs.submit.removeAttribute("disabled");
      this.$refs.cancel.removeAttribute("disabled");
      this.$refs.compress.removeAttribute("disabled");
    },
    copy: function (event) {
      event.preventDefault();
      this.disableControls();

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
        return this.remoteCopy(this.agentId, items, this.compress);
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
            this.enableControls();
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
    remoteCopy: async function (agentId, items, compress) {
      let action = async (overwrite, keep) => {
        await remote_api
          // execute items source and destination checks,
          // the transfer continues in the background
          .copyStart(agentId, items, overwrite, keep, compress)
          .then((res) => {
            this.$store.commit("closeHovers");
            // subscribe to the transfer's status update stream
            let transferID = res.message;
            transfers.create(
              this.$store,
              transferID,
              this.transfers,
              "copy",
              this.agent,
              transfers.prepareItems(items)
            );
            setTimeout(() => {
              buttons.active("transfers");
              buttons.loading("transfers");
            }, 10);
          })
          .catch((e) => {
            this.enableControls();
            buttons.donePromise("transfers").then(() => {
              this.$showError(e);
            });
          });
      };

      let dstItems;
      try {
        dstItems = (await remote_api.fetch(this.agentId, this.dest)).items;
      } catch (e) {
        dstItems = [];
      }
      let conflict = upload.checkConflict(items, dstItems);

      let overwrite = false;
      let keep = false;

      if (conflict) {
        this.$store.commit("showHover", {
          prompt: "remote-replace",
          confirm: (event, option) => {
            overwrite = option == "overwrite";
            keep = option == "keep";

            event.preventDefault();
            this.$store.commit("closeHovers");
            action(overwrite, keep);
          },
        });

        return;
      }

      action(overwrite, keep);
    },
  },
};
</script>
