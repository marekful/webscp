<template>
  <form
    v-if="loginAgent.remote_user !== undefined"
    class="card floating"
    @submit.prevent="login"
  >
    <div class="card-title">
      <h2>{{ $t("prompts.agent.loginTitle") }}</h2>
    </div>
    <div class="card-content">
      <p>
        {{
          $t("prompts.agent.loginText", {
            user: loginAgent.remote_user.name,
            address: loginAgent.host + ":" + loginAgent.port,
          })
        }}
      </p>
      <p>
        <input
          class="input input--block"
          name="password"
          type="password"
          ref="password"
          required="required"
          @keypress.prevent.enter="login"
        />
      </p>
    </div>
    <div class="card-action">
      <button class="button button--flat button--grey" @click.prevent="cancel">
        {{ $t("prompts.agent.cancel") }}
      </button>
      <input
        type="submit"
        class="button button--flat"
        :value="$t('prompts.agent.login')"
      />
    </div>
  </form>
</template>

<script>
import { agents as api } from "@/api";
import { mapState } from "vuex";

export default {
  name: "agent-login",
  data: function () {
    return {
      intervalID: null,
    };
  },
  computed: {
    ...mapState(["loginAgent", "show"]),
  },
  async created() {
    this.$store.commit("setLoginAgent", {
      ...(await api.get(this.loginAgent.id)),
      component: this.loginAgent.component,
    });

    this.intervalID = setInterval(() => {
      if (this.show === null) {
        this.$store.commit("resetLoginAgent");
        this.clearInterval();
      }
    }, 300);
  },
  mounted() {
    let focusInput = () => {
      setTimeout(() => {
        if (this.$refs.password && this.$refs.password.focus) {
          this.$refs.password.focus();
        } else {
          focusInput();
        }
      }, 10);
    };

    focusInput();
  },
  methods: {
    clearInterval() {
      clearInterval(this.intervalID);
      this.intervalID = null;
    },
    login(event) {
      let name = this.loginAgent.remote_user.name;
      let target =
        event.target.length && event.target.length >= 1
          ? event.target[0]
          : event.target;
      let password = target.value;

      api
        .remoteUserLogin(this.loginAgent.id, name, password)
        .then(() => {
          this.$store.commit("showHover", this.loginAgent.component);
          this.clearInterval();
        })
        .catch((err) => {
          if (this.$refs.password) {
            this.$refs.password.select();
          }
          this.$showError(err);
        });
    },
    cancel() {
      let show = this.loginAgent.component;
      this.$store.commit("resetLoginAgent");
      this.clearInterval();
      this.$store.commit("showHover", show);
    },
  },
};
</script>
