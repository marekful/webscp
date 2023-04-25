<template>
  <nav :class="{ active }">
    <template v-if="isLogged">
      <button
        class="action"
        @click="toRoot"
        :aria-label="$t('sidebar.myFiles')"
        :title="$t('sidebar.myFiles')"
      >
        <i class="material-icons">folder</i>
        <span>{{ $t("sidebar.myFiles") }}</span>
      </button>

      <div v-if="user.perm.create && isListing">
        <button
          @click="$store.commit('showHover', 'newDir')"
          class="action"
          :aria-label="$t('sidebar.newFolder')"
          :title="$t('sidebar.newFolder')"
        >
          <i class="material-icons">create_new_folder</i>
          <span>{{ $t("sidebar.newFolder") }}</span>
        </button>

        <button
          @click="$store.commit('showHover', 'newFile')"
          class="action"
          :aria-label="$t('sidebar.newFile')"
          :title="$t('sidebar.newFile')"
        >
          <i class="material-icons">note_add</i>
          <span>{{ $t("sidebar.newFile") }}</span>
        </button>
      </div>

      <div>
        <button
          v-if="isListing && !userHiddenDotfiles"
          class="action"
          @click="toggleDotfiles"
          :aria-label="$t('sidebar.dotfiles')"
          :title="
            this.$store.state.showDotfiles
              ? $t('sidebar.dotfiles_hide')
              : $t('sidebar.dotfiles_show')
          "
        >
          <i class="material-icons">{{ toggleDotfilesIcon }}</i>
          <span>{{ $t("sidebar.dotfiles") }}</span>
        </button>
      </div>

      <div>
        <button
          class="action"
          @click="toSettings"
          :aria-label="$t('sidebar.settings')"
          :title="$t('sidebar.settings')"
        >
          <i class="material-icons">settings_applications</i>
          <span>{{ $t("sidebar.settings") }}</span>
        </button>

        <button
          v-if="canLogout"
          @click="logout"
          class="action"
          id="logout"
          :aria-label="$t('sidebar.logout')"
          :title="$t('sidebar.logout')"
        >
          <i class="material-icons">exit_to_app</i>
          <span>{{ $t("sidebar.logout") }}</span>
        </button>
      </div>
    </template>
    <template v-else>
      <router-link
        class="action"
        to="/login"
        :aria-label="$t('sidebar.login')"
        :title="$t('sidebar.login')"
      >
        <i class="material-icons">exit_to_app</i>
        <span>{{ $t("sidebar.login") }}</span>
      </router-link>

      <router-link
        v-if="signup"
        class="action"
        to="/login"
        :aria-label="$t('sidebar.signup')"
        :title="$t('sidebar.signup')"
      >
        <i class="material-icons">person_add</i>
        <span>{{ $t("sidebar.signup") }}</span>
      </router-link>
    </template>

    <div
      class="credits"
      v-if="
        $router.currentRoute.path.includes('/files/') && !disableUsedPercentage
      "
      style="width: 90%; margin: 2em 2.5em 3em 2.5em"
    >
      <progress-bar :val="usage.usedPercentage" size="small"></progress-bar>
      <br />
      {{ usage.used }} of {{ usage.total }} used
    </div>

    <p class="credits">
      <span>
        <span v-if="disableExternal">WebSCP</span>
        <a
          v-else
          rel="noopener noreferrer"
          target="_blank"
          href="https://github.com/marekful/webscp"
          >WebSCP</a
        >
        <span> {{ version }}</span>
      </span>
      <span>
        <a @click="help">{{ $t("sidebar.help") }}</a>
      </span>
    </p>
  </nav>
</template>

<script>
import { mapState, mapGetters } from "vuex";
import * as auth from "@/utils/auth";
import {
  version,
  signup,
  disableExternal,
  disableUsedPercentage,
  noAuth,
  loginPage,
  authMethod,
} from "@/utils/constants";
import { files as api } from "@/api";
import ProgressBar from "vue-simple-progress";
import prettyBytes from "pretty-bytes";

export default {
  name: "sidebar",
  components: {
    ProgressBar,
  },
  computed: {
    ...mapState(["user"]),
    ...mapGetters(["isLogged", "isListing"]),
    active() {
      return this.$store.state.show === "sidebar";
    },
    signup: () => signup,
    version: () => version,
    disableExternal: () => disableExternal,
    disableUsedPercentage: () => disableUsedPercentage,
    toggleDotfilesIcon() {
      return this.$store.state.showDotfiles ? "visibility" : "visibility_off";
    },
    userHiddenDotfiles() {
      return this.user.hideDotfiles;
    },
    canLogout: () => (!noAuth && loginPage) || authMethod === "oidc",
  },
  asyncComputed: {
    usage: {
      async get() {
        let path = this.$route.path.endsWith("/")
          ? this.$route.path
          : this.$route.path + "/";
        let usageStats = { used: 0, total: 0, usedPercentage: 0 };
        if (this.disableUsedPercentage) {
          return usageStats;
        }
        try {
          let usage = await api.usage(path);
          usageStats = {
            used: prettyBytes(usage.used, { binary: true }),
            total: prettyBytes(usage.total, { binary: true }),
            usedPercentage: Math.round((usage.used / usage.total) * 100),
          };
        } catch (error) {
          this.$showError(error);
        }
        return usageStats;
      },
      default: { used: "0 B", total: "0 B", usedPercentage: 0 },
      shouldUpdate() {
        return this.$router.currentRoute.path.includes("/files/");
      },
    },
  },
  methods: {
    toRoot() {
      this.$router.push({ path: "/files/" }, () => {});
      this.$store.commit("closeHovers");
    },
    toSettings() {
      this.$router.push({ path: "/settings" }, () => {});
      this.$store.commit("closeHovers");
    },
    toggleDotfiles() {
      this.$store.commit("toggleDotfiles", true);
    },
    help() {
      this.$store.commit("showHover", "help");
    },
    logout: auth.logout,
  },
};
</script>
