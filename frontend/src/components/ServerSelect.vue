<template>
  <div v-if="serverList.length > 1" class="server-select">
    <div class="section-title">
      <div v-if="showSettings" class="settings-content"></div>
      <span class="settings">
        <i class="material-icons" @click="settings">settings</i>
      </span>
      {{ $t("prompts.agent.selectServer") }}
    </div>
    <custom-select
      :options="serverList"
      :default="{ title: { text: 'Local' } }"
      class="select"
      @input="changed($event)"
      :tabindex="selectedIndex"
    ></custom-select>
  </div>
</template>

<script>
import { mapState } from "vuex";
import { agents } from "@/api";
import CustomSelect from "@/components/CustomSelect";

export default {
  name: "server-select",
  components: { CustomSelect },
  data: function () {
    return {
      servers: {},
      serverList: [],
      selectedIndex: 0,
      showSettings: false,
    };
  },
  computed: {
    ...mapState(["req", "user", "loginAgent"]),
  },
  mounted() {
    this.fillOptions();

    if (this.loginAgent && this.loginAgent.id) {
      setTimeout(() => {
        this.selectedIndex = this.getServerIndexByID(this.loginAgent.id);
        this.$store.commit("resetLoginAgent");
      }, 200);
    } else {
      this.$emit("update:selected", { id: 0 });
    }
  },
  methods: {
    settings() {
      this.showSettings = !this.showSettings;
    },
    getServerIndexByID(id) {
      for (let index = 1; index <= this.serverList.length; index++) {
        let server = this.servers[index];
        if (server.id === id) return server.index;
      }
      return 0;
    },
    changed(val) {
      let agent = { id: 0 };
      if (val.index in this.servers) {
        agent = this.servers[val.index];
      }
      this.$emit("update:selected", agent);
    },
    async fillOptions() {
      let servers = (await agents.getAll()) || [];
      this.serverList = [{ title: { text: "Local" } }];

      for (let index = 0; index < servers.length; index++) {
        let server = servers[index];
        let label = `${server.host}:${server.port}`;
        this.servers[index + 1] = {
          id: server.id,
          branding: server.branding,
          user: { id: this.user.id },
          host: server.host,
          port: server.port,
          index: index + 1,
        };
        const option = {
          title: {
            text: `${server.branding} (${server.remote_user.name})`,
          },
          body: {
            text: label,
          },
        };
        this.serverList.push(option);
      }
    },
  },
};
</script>

<style>
*,
*::before,
*::after {
  box-sizing: border-box;
}

.server-select {
  border-bottom: 1px solid #aaa;
  padding: 0 1em 1.25em 1em;
  margin: 0 -1em;
}

h1 {
  text-align: center;
}

div.section-title {
  margin: 0 0 1em 0;
}

.section-title .settings {
  float: right;
  margin: 0.15em;
  opacity: 0.4;
  cursor: pointer;
}

.section-title .settings:hover {
  opacity: 0.75;
}

.section-title .settings i {
  font-size: initial;
}

.section-title .settings-content {
  float: right;
  position: absolute;
  right: 2.5em;
  border: 1px solid var(--card-border);
  padding: 1em;
  z-index: 1;
  background-color: var(--card-title-background);
  box-shadow: 0 2px 2px 0 rgba(0, 0, 0, 0.14), 0 1px 5px 0 rgba(0, 0, 0, 0.12),
    0 3px 1px -2px rgba(0, 0, 0, 0.2);
}

.section-title .settings-content label {
  margin-left: 0.5em;
}

.server-select .custom-select > .selected {
  border: 1px solid var(--dark-blue);
  background-color: var(--distinct-background);
  color: var(--textPrimary);
}

.server-select .custom-select > .selected.open {
  border: 1px solid var(--dark-blue);
  border-radius: 6px 6px 0 0;
}

.server-select .custom-select > .selected:after {
  border: 5px solid var(--dark-blue);
  border-color: var(--dark-blue) transparent transparent transparent;
}

.server-select .custom-select .items {
  border-right: 1px solid var(--dark-blue);
  border-left: 1px solid var(--dark-blue);
  border-bottom: 1px solid var(--dark-blue);
  color: var(--surfaceSecondary);
  border-radius: 0px 0px 6px 6px;
  background-color: var(--distinct-background);
}

.server-select .custom-select .items .selected {
  color: var(--dark-blue);
  border: 1px solid var(--grey-blue);
  border-width: 1px 0;
  background-color: var(--distinct-hover);
}

.server-select .custom-select .items > div {
  color: var(--textPrimary);
}

.server-select .custom-select .option {
  color: var(--card-title-color);
}

.server-select .custom-select .option .title {
  font-weight: bold;
  line-height: initial;
  padding: 0.5em 0 0.5em 0;
}

.server-select .custom-select .option .body {
  font-size: small;
  line-height: initial;
  padding: 0 0 0.5em 0;
  color: var(--card-text-color);
}

.server-select .custom-select .items > div:first-child {
  margin-top: 0.5em;
}

.server-select .custom-select .items > div:last-child {
  margin-bottom: 0.5em;
}

.server-select .custom-select .items .option:hover {
  background-color: var(--distinct-hover);
  color: var(--dark-blue);
}
</style>
