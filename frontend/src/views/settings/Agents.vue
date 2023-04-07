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
              <th></th>
              <th v-if="user.perm.admin">{{ $t("settings.agent.owner") }}</th>
              <th>{{ $t("settings.agent.host") }}</th>
              <th>{{ $t("settings.agent.port") }}</th>
              <th>{{ $t("settings.agent.user") }}</th>
              <th :title="$t('settings.agent.agentVersionHint')">
                {{ $t("settings.agent.agentVersion") }}
              </th>
              <th :title="$t('settings.agent.latencyHint')">
                {{ $t("settings.agent.latency") }}
              </th>
              <th></th>
            </tr>

            <tr v-for="agent in agents" :key="agent.id">
              <td class="status">
                <div :class="'status-' + (agent.status || 'loading')">
                  <i v-if="agent.status === 'online'" class="material-icons">
                    task_alt
                  </i>
                  <i
                    v-else-if="agent.status === 'error'"
                    class="material-icons"
                  >
                    cloud_off_outlined
                  </i>
                  <i v-else class="material-icons">help_outline</i>
                </div>
              </td>
              <td v-if="user.perm.admin">{{ agent.userID }}</td>
              <td>{{ agent.host }}</td>
              <td>{{ agent.port }}</td>
              <td>{{ agent.remote_user.name }}</td>
              <td v-if="!agent.error" class="version">{{ agent.version }}</td>
              <td v-else class="version">
                <div class="error">{{ agent.error }}</div>
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
    ...mapState(["loading", "user"]),
  },
  async created() {
    this.setLoading(true);
  },
  async mounted() {
    try {
      this.agents = await api.getAll();
    } catch (e) {
      this.error = e;
      this.agents = [];
    } finally {
      this.setLoading(false);
      if (!this.agents) {
        this.agents = [];
      }
    }

    for (let idx = 0; idx < this.agents.length; idx++) {
      fetch(`/api/agents/${this.agents[idx].id}/version`)
        .then((response) => {
          return response.text();
        })
        .then((json) => {
          let v = JSON.parse(json);
          let a = this.agents[idx];
          a.latency = v.latency;
          a.version = v.version;
          a.status = "online";
          if (v.error) {
            a.error = v.error;
            a.status = "error";
          }
          this.agents.splice(idx, 1, a);
        });
    }
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
.version {
  max-width: 18em;
}
td.status {
  max-width: 3rem;
  width: 3rem;
}
.error {
  word-wrap: break-word;
  word-break: break-word;
  font-size: 80%;
  background-color: var(--moon-grey);
  padding: 0.5em;
}
.status-online {
  color: var(--icon-blue);
}
.status-error,
.status-loading {
  color: var(--mid-grey);
}
.status-online,
.status-error,
.status-loading,
.status-online i,
.status-error i,
.status-loading i {
  font-size: 1.25rem;
  max-width: 1.5em;
  margin-top: 0.1em;
}
</style>
