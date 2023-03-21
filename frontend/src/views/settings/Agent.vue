<template>
  <errors v-if="error" :errorCode="error.status" />
  <div class="row" v-else-if="!loading">
    <div class="column">
      <form @submit="save" class="card">
        <div class="card-title">
          <h2 v-if="agent.id === 0">
            {{ $t("settings.agent.newConnection") }}
          </h2>
          <h2 v-else>{{ $t("settings.agent.remoteAgent") }}</h2>
        </div>

        <div class="card-content">
          <agent-form :agent.sync="agent" :isNew="isNew" />
        </div>

        <div class="card-action">
          <button
            v-if="!isNew"
            @click.prevent="deletePrompt"
            type="button"
            class="button button--flat button--red"
            :aria-label="$t('buttons.delete')"
            :title="$t('buttons.delete')"
          >
            {{ $t("buttons.delete") }}
          </button>
          <input
            v-if="isNew"
            class="button button--flat"
            type="submit"
            :value="$t('buttons.connectAndSave')"
          />
        </div>
      </form>
    </div>

    <div v-if="$store.state.show === 'deleteAgent'" class="card floating">
      <div class="card-content">
        <p>Are you sure you want to delete this agent connection?</p>
      </div>

      <div class="card-action">
        <button
          class="button button--flat button--grey"
          @click="closeHovers"
          v-focus
          :aria-label="$t('buttons.cancel')"
          :title="$t('buttons.cancel')"
        >
          {{ $t("buttons.cancel") }}
        </button>
        <button class="button button--flat" @click="deleteAgent">
          {{ $t("buttons.delete") }}
        </button>
      </div>
    </div>
  </div>
</template>

<script>
import { mapState, mapMutations } from "vuex";
import { agents as api } from "@/api";
import AgentForm from "@/components/settings/AgentForm";
import Errors from "@/views/Errors";

export default {
  name: "agent",
  components: {
    AgentForm,
    Errors,
  },
  data: () => {
    return {
      error: null,
      originalAgent: null,
      agent: {},
    };
  },
  created() {
    this.fetchData();
  },
  computed: {
    isNew() {
      return this.$route.path === "/settings/agents/new";
    },
    ...mapState(["loading"]),
  },
  methods: {
    ...mapMutations(["closeHovers", "showHover", "setLoading"]),
    async fetchData() {
      this.setLoading(true);

      try {
        if (this.isNew) {
          this.agent = {
            host: "",
            port: "",
            secret: "",
            id: 0,
          };
        } else {
          const id = this.$route.params.pathMatch;
          this.agent = { ...(await api.get(id)) };
        }
      } catch (e) {
        this.error = e;
      } finally {
        this.setLoading(false);
      }
    },
    deletePrompt() {
      this.showHover("deleteAgent");
    },
    async deleteAgent(event) {
      event.preventDefault();

      try {
        await api.remove(this.agent.id);
        this.$router.push({ path: "/settings/agents" });
        this.$showSuccess(this.$t("settings.agent.connectionDeleted"));
      } catch (e) {
        e.message === "403"
          ? this.$showError(this.$t("errors.forbidden"), false)
          : this.$showError(e);
      }
    },
    async save(event) {
      event.preventDefault();
      let agent = {
        ...this.originalAgent,
        ...this.agent,
      };

      try {
        if (this.isNew) {
          const loc = await api.create(agent);
          this.$router.push({ path: loc });
          this.$showSuccess(this.$t("settings.agent.connectionCreated"));
        } else {
          await api.update(agent);

          this.$showSuccess(this.$t("settings.agen.connectionUpdated"));
        }
      } catch (e) {
        this.$showError(e);
      }
    },
  },
};
</script>
