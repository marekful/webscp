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
      let agent_id = 0;
      if (val in this.servers) {
        agent_id = this.servers[val].id;
      }
      this.$emit("update:selected", agent_id);
    },
    async fillOptions() {
      let servers = await agents.getAll();
      this.serverList = ["Local"];

      for (let server of servers) {
        let key = server.host + ":" + server.port;
        this.servers[key] = {
          id: server.id,
          host: server.host,
          port: server.port,
        };
        this.serverList.push(key);
      }
    },
    /*select: function (event) {
      // If the element is already selected, unselect it.
      if (this.selected === event.currentTarget.dataset.url) {
        this.selected = null;
        this.$emit("update:selected", this.current);
        return;
      }

      // Otherwise select the element.
      this.selected = event.currentTarget.dataset.url;
      this.$emit("update:selected", this.selected);
    },*/
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
