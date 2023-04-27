<template>
  <errors v-if="error" :errorCode="error.status" />
  <div class="row" v-else-if="!loading">
    <div class="column">
      <div class="card" @submit.prevent="save">
        <div class="card-title">
          <h2>{{ $t("settings.agent.accessTokens") }}</h2>
        </div>
        <div class="card-content">
          <div>{{ $t("settings.agent.accessTokenHint") }}</div>
          <div v-if="tokens.length > 0">
            <div class="token" v-for="(token, index) in tokens" :key="index">
              <div class="info">
                <div>
                  <i class="material-icons">info</i>
                </div>
                <div>
                  {{ $t("settings.agent.tokenHint") }}
                </div>
              </div>
              <p>
                <label>{{ $t("settings.agent.accessToken") }}</label>
                <textarea
                  readonly
                  v-model="token.token"
                  @focus="$event.target.select()"
                ></textarea>
              </p>
              <p>
                <label>{{ $t("settings.agent.validUntil") }}</label>
                <span>{{ new Date(token.valid_until * 1000) }}</span>
              </p>
            </div>
          </div>
        </div>

        <div class="card-action">
          <button class="button button--flat" @click="token">
            {{ $t("settings.agent.generateAccessToken") }}
          </button>
        </div>
      </div>
    </div>

    <div class="column column-w">
      <form class="card">
        <div class="card-title">
          <h2>{{ $t("settings.agent.connections") }}</h2>
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
              <td>{{ agent.host }}</td>
              <td>{{ agent.port }}</td>
              <td>{{ agent.remote_user.name }}</td>
              <td v-if="!agent.error" class="version">{{ agent.version }}</td>
              <td v-else class="version" :colspan="agent.error ? '2' : ''">
                <div class="error">{{ agent.error }}</div>
              </td>
              <td v-if="!agent.error">{{ agent.latency }}</td>
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
  name: "agents",
  components: {
    Errors,
  },
  data: function () {
    return {
      error: null,
      agents: [],
      tokens: [],
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
    async token() {
      await api
        .getTemporaryAccessToken()
        .then((token) => this.tokens.push(token));
    },
  },
};
</script>

<style scoped>
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
  background-color: var(--distinct-background);
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

.card .token {
  border-bottom: 3px solid var(--moon-grey);
  margin: 1.5em 0;
}

.card .token textarea {
  width: 100%;
  background-color: var(--moon-grey);
  height: 4em;
  border: 1px solid var(--icon-blue);
  padding: 0.5em;
  border-radius: 2px;
  color: var(--card-text-color);
}

.card .token p label {
  font-size: initial;
  margin: 1.25em 0 0.75em 0;
}

.card .token .info {
  background-color: var(--distinct-background);
  padding: 0.75em;
  border-radius: 2px;
  border: 1px solid var(--card-border);
  color: var(--card-text-color);
}

.card .token .info i {
  font-size: 1.25em;
  vertical-align: bottom;
  color: var(--icon-blue);
}

.card .token .info > div:first-child {
  float: left;
  margin-right: 0.5em;
}
</style>
