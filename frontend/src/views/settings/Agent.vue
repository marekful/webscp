<template>
  <errors v-if="error" :errorCode="error.status" />
  <div class="row" v-else-if="!loading">
    <div class="column">
      <form ref="form" @keypress.prevent.enter="submit" class="card">
        <div class="card-title">
          <h2 v-if="agent.id === 0">
            {{ $t("settings.agent.newConnection") }}
          </h2>
          <h2 v-else>{{ $t("settings.agent.editConnection") }}</h2>
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
          <button
            v-if="!isNew"
            ref="submit"
            @click="save"
            type="button"
            class="button button--flat"
            :aria-label="$t('buttons.save')"
            :title="$t('buttons.save')"
          >
            {{ $t("buttons.save") }}
          </button>
          <stateful-button
            v-if="isNew"
            ref="submit"
            :handler="save"
            class-name="button button--flat"
            label-tr="buttons.connectAndSave"
            title-tr="buttons.connectAndSave"
          ></stateful-button>
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
import StatefulButton from "@/components/StatefulButton";
import Errors from "@/views/Errors";

export default {
  name: "agent",
  components: {
    StatefulButton,
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
            branding: "",
            secret: "",
            remote_user: {
              name: "",
              password: "",
            },
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
    submit(event) {
      const submit = this.$refs.submit;
      submit && submit.$el && submit.$el.click(event);
      submit && submit.click && submit.click(event);
    },
    async save(event) {
      event.preventDefault();
      let agent = {
        ...this.originalAgent,
        ...this.agent,
      };

      try {
        const params = this.isNew ? null : ["branding"];
        const method = this.isNew ? "create" : "update";
        const message = this.isNew ? "Created" : "Updated";
        const loc = await api[method](agent, params);
        this.$showSuccess(this.$t(`settings.agent.connection${message}`));
        await this.$router.push({ path: loc });
      } catch (e) {
        this.$showError(e);
      }
    },
  },
};
</script>
