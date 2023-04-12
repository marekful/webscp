<template>
  <div class="agent-form">
    <p>
      <label for="host">{{ $t("settings.agent.hostLabel") }}</label>
      <input
        v-if="isNew"
        class="input input--block"
        type="text"
        v-model="agent.host"
        id="host"
        required="required"
      />
      <code v-else>
        {{ agent.host }}
      </code>
    </p>

    <p>
      <label for="port">{{ $t("settings.agent.port") }}</label>
      <input
        v-if="isNew"
        class="input input--block"
        type="number"
        placeholder="7022"
        v-model="agent.port"
        id="port"
        required="required"
      />
      <code v-else>
        {{ agent.port }}
      </code>
    </p>

    <p v-if="isNew">
      <label for="secret">{{ $t("settings.agent.accessToken") }}</label>
      <input
        class="input input--block"
        type="password"
        id="secret"
        v-model="agent.secret"
        required="required"
      />
    </p>

    <p>
      <label for="remote-user-name">
        {{ $t("settings.agent.remoteUserName") }}
      </label>
      <input
        v-if="isNew"
        class="input input--block"
        type="text"
        id="remote-user-name"
        v-model="agent.remote_user.name"
        required="required"
      />
      <code v-else>
        {{ agent.remote_user.name }} ({{ agent.remote_user.id }})
      </code>
    </p>

    <p v-if="!isNew">
      <label>{{ $t("settings.agent.remotePath") }}</label>
      <code>
        {{ agent.remote_user.root }}
      </code>
    </p>

    <p v-if="isNew">
      <label for="remote-user-password">
        {{ $t("settings.agent.remoteUserPassword") }}
      </label>
      <input
        class="input input--block"
        type="password"
        id="remote-user-password"
        v-model="agent.remote_user.password"
        required="required"
      />
    </p>
  </div>
</template>

<script>
export default {
  name: "agent",
  data: () => {
    return {};
  },
  components: {},
  props: ["agent", "isNew"],
  computed: {},
};
</script>

<style>
.dashboard .card-content .agent-form p label {
  font-size: initial;
  margin-bottom: 0.2em;
  display: block;
  font-weight: 500;
}
.dashboard .card-content .agent-form code {
  display: inline-block;
  margin-top: 0.25em;
}
</style>
