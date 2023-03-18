<template>
  <errors v-if="error" :errorCode="error.status" />
  <div class="row" v-else-if="!loading">
    <div class="column">
      <form class="card" @submit.prevent="save">
        <div class="card-title">
          <h2>{{ $t("settings.agent.agentStatus") }}</h2>
        </div>

        <div class="card-action">
          <input
            class="button button--flat"
            type="submit"
            :value="$t('buttons.update')"
          />
        </div>
      </form>
    </div>

    <div class="column column-w">
      <form class="card" @submit.prevent="save">
        <div class="card-title">
          <h2>{{ $t("settings.agent.remoteAgents") }}</h2>
          <router-link to="/settings/agents/new"
            ><button class="button">
              {{ $t("buttons.new") }}
            </button></router-link
          >
        </div>

        <div class="card-content full">
          <table>
            <tr>
              <th>{{ $t("settings.agent.address") }}</th>
              <th>{{ $t("settings.agent.status") }}</th>
              <th>
                {{ $t("settings.agent.agentVersion") }}<br />
                <small>{{ $t("settings.agent.agentVersionHint") }}</small>
              </th>
              <th>{{ $t("settings.agent.ping") }}<br />
                <small>{{ $t("settings.agent.pingHint") }}</small>
              </th>
              <th></th>
            </tr>

            <tr v-for="agent in agents" :key="agent.id">
              <td>{{ agent.host }}:{{ agent.port }}</td>
              <td>{{ agent.status }}</td>
              <td v-if="!agent.error">{{ agent.version }}</td>
              <td v-else>
                <small>{{ agent.error }}</small>
              </td>
              <td>
                <small>{{ agent.latency }}</small>
              </td>
              <td class="small">
                <router-link :to="'/settings/agents/' + agent.id"
                  ><i class="material-icons">mode_edit</i></router-link
                >
              </td>
            </tr>
          </table>
        </div>
      </form>
    </div>
  </div>
</template>

<script>
import { mapState, mapMutations } from "vuex";
import { agents as api } from "@/api";
import Errors from "@/views/Errors";

export default {
  name: "settings",
  components: {
    Errors,
  },
  data: function () {
    return {
      error: null,
      agents: [],
    };
  },
  computed: {
    ...mapState(["loading"]),
  },
  async created() {
    this.setLoading(true);

    try {
      this.agents = await api.getAll();
    } catch (e) {
      this.error = e;
    } finally {
      this.setLoading(false);
    }
    const promises = [];
    this.agents.forEach(async (a) => {
      promises.push(fetch(`/api/agents/${a.id}/version`)
          .then((response) => {
            return response.text();
          })
          .then((json) => {
            let v = JSON.parse(json);
            a.latency = v.latency
            a.version =  v.version;
            a.status = "online"
            if (v.error) {
              a.error = v.error;
              a.status = "error"
            }
            return a;
          }));
    });

    Promise.all(promises).then((agents) => {
      this.agents = agents;
    })
  },
  methods: {
    ...mapMutations(["setLoading"]),
    capitalize(name, where = "_") {
      if (where === "caps") where = /(?=[A-Z])/;
      let splitted = name.split(where);
      name = "";

      for (let i = 0; i < splitted.length; i++) {
        name +=
          splitted[i].charAt(0).toUpperCase() + splitted[i].slice(1) + " ";
      }

      return name.slice(0, -1);
    },
    async save() {
      let agent = {
        ...this.agent,
      };

      try {
        await api.update(agent);
        this.$showSuccess(this.$t("settings.settingsUpdated"));
      } catch (e) {
        this.$showError(e);
      }
    },
  },
};
</script>

<style>
  th small {
    font-size: 70%;
  }
</style>