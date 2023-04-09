<template>
  <div id="server-select">
    <div class="section-title">{{ $t("prompts.selectServer") }}</div>
    <custom-select
      :options="serverList"
      :default="'Local'"
      class="select"
      @input="changed($event)"
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
    };
  },
  computed: {
    ...mapState(["req", "user"]),
  },
  mounted() {
    this.fillOptions();
  },
  methods: {
    changed(val) {
      let agent = { id: 0 };
      if (val.index in this.servers) {
        agent = this.servers[val.index];
      }
      this.$emit("update:selected", agent);
    },
    async fillOptions() {
      let servers = await agents.getAll();
      this.serverList = ["Local"];

      for (let index = 0; index < servers.length; index++) {
        let server = servers[index];
        let label = `${server.host}:${server.port} (${server.remote_user.name})`;
        this.servers[index + 1] = {
          id: server.id,
          user: { id: this.user.id },
          host: server.host,
          port: server.port,
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

#app {
  max-width: 400px;
  margin: 0 auto;
  line-height: 1.4;
  font-family: "Avenir", Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

#server-select {
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
</style>
