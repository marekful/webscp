<template>
  <div class="card transfers top">
    <section class="card-content">
      <div v-for="(transfer, index) in transfers" :key="index" class="transfer">
        <div class="title ellipse" :title="transfer.title">
          <i class="material-icons">content_copy_outlined</i>
          {{ transfer.title }}
        </div>
        <div class="content">
          <div
            :class="transfer.cancelable ? 'remove' : 'remove disabled'"
            @click="handleCancel"
            :data-id="transfer.transferID"
            :title="
              transfer.pending ? $t('transfer.cancel') : $t('transfer.remove')
            "
          >
            <i class="material-icons">
              {{ transfer.cancelable ? "cancel" : "highlight_off" }}
            </i>
          </div>
          <div
            :class="{
              icon: true,
              iconError: transfer.error === true || transfer.canceled === true,
              iconSuccess:
                transfer.pending === false &&
                transfer.error === false &&
                transfer.canceled === false,
            }"
          >
            <i class="material-icons">{{ transfer.icon }}</i>
          </div>
          <div v-if="transfer.error !== true" class="status">
            <span>
              {{ transfer.status }}
              <span
                v-if="transfer.stats && transfer.stats.progress.length > 0"
                class="stats"
              >
                <span
                  v-if="
                    transfer.uploading === true ||
                    transfer.canceled === true ||
                    transfer.error === true
                  "
                >
                  <span>{{ transfer.stats.progress[0] }}</span>
                  <small class="frac">.{{ transfer.stats.progress[1] }}</small>
                  <small class="unit">{{ transfer.stats.progress[2] }}</small>
                  <small>of</small>
                </span>
                <span>{{ transfer.stats.total[0] }}</span>
                <small class="frac">.{{ transfer.stats.total[1] }}</small>
                <small class="unit">{{ transfer.stats.total[2] }}</small>
              </span>
            </span>
          </div>
          <div v-else class="error">
            <span>{{ transfer.status }}</span>
          </div>
          <div class="details">
            <div class="icon" @click="showDetails(transfer)">
              <i class="material-icons">arrow_drop_down_circle</i>
              <span class="label">{{ $t("transfer.showDetails") }}</span>
            </div>
            <ul v-if="transfer.showDetails === true" class="content">
              <li v-for="(item, index) in transfer.items" :key="index">
                <span class="path">{{ item.from }}</span>
                <span class="name">{{ item.name }}</span>
                <i class="material-icons">east</i>
                <span class="to">{{ item.to }}</span>
              </li>
            </ul>
          </div>
        </div>
      </div>
      <div v-if="transfers.length === 0" class="transfer no-content">
        No transfers
      </div>
    </section>

    <section class="card-action">
      <button
        class="button button--flat"
        @click="$store.commit('closeHovers')"
        :aria-label="$t('buttons.cancel')"
        :title="$t('buttons.cancel')"
      >
        {{ $t("buttons.close") }}
      </button>
    </section>
  </div>
</template>

<script>
import { mapState } from "vuex";
import transfers from "@/utils/transfers";
import { remote_files } from "@/api";
import i18n from "@/i18n";

export default {
  name: "transfers",
  data: function () {
    return {};
  },
  computed: {
    ...mapState(["req", "user", "transfers"]),
  },
  methods: {
    cancel: function () {
      this.$store.commit("closeHovers");
    },
    handleCancel: function (event) {
      let transferID = event.target.parentNode.dataset["id"];
      let transfer = transfers.get(this.transfers, transferID);

      if (!transfer || !transfer.cancelable) {
        return;
      }

      let cancel = () => {
        if (!transfer.pending) {
          transfers.remove(this.$store, transferID);
          transfers.setButtonActive(this.transfers);

          return;
        }

        transfer.sseClient && transfer.sseClient.close();
      };

      let update = () => {
        transfers.update(this.$store, {
          transferID,
          canceled: true,
          pending: false,
          icon: "highlight_off",
          status: i18n.t("transfer.canceled"),
        });
      };

      let error = (e) => {
        if (e.message.indexOf("403 Forbidden") > -1) {
          this.$showError(e);
        } else {
          cancel();
        }
      };

      remote_files
        .cancelTransfer(transfer.agent.id, transferID)
        .then(cancel)
        .then(update)
        .catch(error)
        .finally(() => transfers.setButtonActive(this.transfers));
    },
    showDetails: function (transfer) {
      transfers.update(this.$store, {
        transferID: transfer.transferID,
        showDetails: !transfer.showDetails,
      });
    },
  },
};
</script>

<style>
.card.transfers {
  position: fixed;
  top: 4.2em;
  z-index: 99999;
  color: var(--card-text-color);
  max-width: 30em;
  max-height: 95%;
  animation: 0.1s show forwards;
}

.card.transfers.top {
  right: 5%;
  width: 90%;
}

@media (max-width: 736px) {
  .card.transfers.top {
    right: 0;
    width: 100%;
  }
}

.transfer {
  border-width: 0 0 1px 0;
  border-style: solid;
  border-color: var(--card-border);
  min-height: 5em;
}

.transfer:first-child {
  border-width: 1px 0;
}

.ellipse {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.transfer > .title {
  font-size: 110%;
  padding: 0.75em 1em;
  color: var(--card-title-color);
  border-bottom: 1px solid var(--card-border-light);
  background-color: var(--card-title-background);
  cursor: default;
}

.transfer > .content {
  padding: 1em;
}

.transfer > .title > i {
  font-size: 1em;
  max-width: 1em;
  margin: 0 0.4em 0 0;
  color: var(--dark-grey);
  vertical-align: middle;
  opacity: 0.8;
}

.transfer .error,
.transfer .status {
  display: table;
}

.transfer .status {
  line-height: 2.5em;
  font-size: 105%;
  text-transform: capitalize;
  font-weight: bold;
}

.transfer .error {
  color: var(--dark-red);
  background: transparent;
}

.transfer .stats {
  margin: 0 0 0 0.5em;
  font-style: italic;
  font-weight: normal;
}

.transfer .stats span {
  margin: 0 0 0 0.25em;
}

.transfer .stats small.frac {
  font-size: 75%;
  margin: 0;
}

.transfer .stats small.unit {
  font-size: 85%;
  margin: 0 0.25em 0 0;
}

.transfer > .content > .icon,
.transfer > .content > .icon-error {
  float: left;
  max-width: 2em;
  margin: 0 0.75em 0 0;
  padding: 0.25em 0 0 0;
}

.transfer > .content > .icon i {
  font-size: 2rem;
}

.transfer > .content > .icon:not(.iconError):not(.iconSuccess) i {
  color: #546e7a;
  animation-duration: 5s;
  animation-name: change-color;
  animation-iteration-count: infinite;
  animation-direction: alternate;
}

@keyframes change-color {
  from {
    color: #546e7a;
  }

  to {
    color: var(--icon-blue);
  }
}

.transfer .iconError i {
  color: var(--dark-red);
}

.transfer .iconSuccess i {
  color: var(--icon-green);
}

.transfer.no-content {
  color: var(--mid-grey);
  padding: 2em;
}

.transfer > .content > .remove {
  float: right;
  margin: 0 0 0 0.5em;
  padding: 1em 0 0 0;
  opacity: 0.4;
  cursor: pointer;
}

.transfer .remove:hover {
  opacity: 0.8;
}

.transfer .remove.disabled {
  opacity: 0.3;
  cursor: default;
}

.transfer > .content > .details {
  margin-top: 0.25em;
  font-size: 90%;
}

.transfer > .content > .details .path {
  color: var(--mid-grey);
}

.transfer > .content > .details .name {
  font-weight: bold;
  margin-right: 0.25em;
}

.transfer > .content > .details .to {
  margin-left: 1.25em;
}

.transfer > .content > .details > .content i {
  position: absolute;
  font-size: 1rem;
  width: 18px;
}

.transfer > .content > .details > .icon {
  text-align: center;
  margin-bottom: -0.5em;
  cursor: pointer;
  opacity: 0.5;
}

.transfer > .content > .details > .icon:hover {
  opacity: 0.8;
}

.transfer > .content > .details > .icon > i {
  font-size: 1em;
  vertical-align: middle;
}

.transfer > .content > .details > .icon > .label {
  font-size: 90%;
  margin: 0 0 0 0.25em;
}

.transfer > .content > .details > ul.content {
  margin: 1.5em 0 0 0;
  padding: 0 0 0 1em;
}

.transfer > .content > .details > ul > li {
  margin: 0 0 0.25em 0;
}

section.card-action {
  padding: 0.5em;
}
</style>
