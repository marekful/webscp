<template>
  <div class="agent-form">
    <p v-if="!isNew">
      <label for="name">{{ $t("settings.agent.connectionName") }}</label>
      <input
        class="input input--block"
        type="text"
        v-model="agent.branding"
        id="name"
        name="name"
        required="required"
      />
    </p>
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
      <label for="port">{{ $t("settings.agent.agentPort") }}</label>
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

    <p v-if="!isNew">
      <label>{{ $t("settings.agent.remoteUser") }}</label>
      <code>{{ agent.remote_user.name }} ({{ agent.remote_user.id }})</code>
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
