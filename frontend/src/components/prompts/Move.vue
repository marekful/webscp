<template>
  <div class="card floating">
    <div class="card-title">
      <h2>{{ $t("prompts.move") }}</h2>
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
      <stateful-button
        :handler="move"
        :disabled="$route.path === dest && modeLocal"
        class-name="button button--flat"
        label-tr="buttons.move"
        title-tr="buttons.move"
      ></stateful-button>
    </div>
  </div>
</template>

<script>
import { mapState } from "vuex";
import FileList from "./FileList";
import ServerSelect from "../ServerSelect";
import StatefulButton from "@/components/StatefulButton.vue";
import { files as api } from "@/api";
import buttons from "@/utils/buttons";
import * as upload from "@/utils/upload";

export default {
  name: "move",
  components: { FileList, ServerSelect, StatefulButton },
  data: function () {
    return {
      current: window.location.pathname,
      dest: null,
      agentId: null,
      modeLocal: true,
    };
  },
  computed: mapState(["req", "selected"]),
  methods: {
    changeServer: function (val) {
      let id = val.id;
      if (id === 0) {
        this.agentId = 0;
        this.modeLocal = true;
      } else {
        this.agentId = id;
        this.agent = val;
        this.modeLocal = false;
      }
    },
    move: async function (event) {
      event.preventDefault();
      let items = [];

      if (!this.modeLocal) {
        const err = new Error("Remote move is not yet implemented");
        this.$showError(err);
        return;
      }

      for (let item of this.selected) {
        items.push({
          from: this.req.items[item].url,
          to: this.dest + encodeURIComponent(this.req.items[item].name),
          name: this.req.items[item].name,
        });
      }

      let action = async (overwrite, rename) => {
        buttons.loading("move");

        await api
          .move(items, overwrite, rename)
          .then(() => {
            buttons.success("move");
            this.$router.push({ path: this.dest });
          })
          .catch((e) => {
            buttons.done("move");
            this.$showError(e);
          });
      };

      let dstItems = (await api.fetch(this.dest)).items;
      let conflict = upload.checkConflict(items, dstItems);

      let overwrite = false;
      let rename = false;

      if (conflict) {
        this.$store.commit("showHover", {
          prompt: "replace-rename",
          confirm: (event, option) => {
            overwrite = option === "overwrite";
            rename = option === "rename";

            event.preventDefault();
            this.$store.commit("closeHovers");
            action(overwrite, rename);
          },
        });

        return;
      }

      action(overwrite, rename);
    },
  },
};
</script>
