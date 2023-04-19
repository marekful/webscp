<template>
  <div v-if="serverList.length > 1" class="server-select">
    <div class="section-title">{{ $t("prompts.selectServer") }}</div>
    <custom-select
      :options="serverList"
      :default="'Local'"
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
      this.serverList = ["Local"];

      for (let index = 0; index < servers.length; index++) {
        let server = servers[index];
        let label = `${server.host}:${server.port} (${server.remote_user.name})`;
        this.servers[index + 1] = {
          id: server.id,
          user: { id: this.user.id },
          host: server.host,
          port: server.port,
          index: index + 1,
        };
        this.serverList.push(label);
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
  background-color: var(--moon-grey);
}

.server-select .custom-select .items .selected {
  color: var(--dark-blue);
  border: 1px solid var(--dark-blue);
  border-width: 1px 0;
  background-color: var(--distinct-hover);
}

.server-select .custom-select .items div {
  color: var(--textPrimary);
}

.server-select .custom-select .items div:first-child {
  margin-top: 0.5em;
}

.server-select .custom-select .items div:last-child {
  margin-bottom: 0.5em;
}

.server-select .custom-select .items div:hover {
  background-color: var(--distinct-hover);
  color: var(--dark-blue);
}
</style>
