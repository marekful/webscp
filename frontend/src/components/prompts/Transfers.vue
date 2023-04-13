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
            @click="handleCancel(transfer)"
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
              'icon-error': transfer.error === true,
              'icon-canceled': transfer.canceled === true,
              'icon-success':
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
              <span v-if="transfer.showDetails === false">
                <i class="material-icons">keyboard_arrow_down</i>
                <span class="label">{{ $t("transfer.showDetails") }}</span>
              </span>
              <span v-else>
                <i class="material-icons">keyboard_arrow_up</i>
                <span class="label">{{ $t("transfer.hideDetails") }} </span>
              </span>
            </div>
            <div v-if="transfer.showDetails">
              <div class="summary">
                <div class="title">
                  <span>{{ $t(`transfer.continuous.${transfer.action}`) }}</span
                  >&nbsp;
                  <a href="#" @click.prevent="showItems(transfer)">
                    <span v-if="numFiles(transfer) > 0">
                      {{ numFiles(transfer) }}
                      {{
                        $t(`transfer.file${numFiles(transfer) > 1 ? "s" : ""}`)
                      }}
                    </span>
                    <span v-if="numDirs(transfer) > 0">
                      <span v-if="numFiles(transfer) > 0">
                        {{ $t("transfer.and") }}
                      </span>
                      {{ numDirs(transfer) }}
                      {{
                        $t(
                          `transfer.director${
                            numDirs(transfer) > 1 ? "ies" : "y"
                          }`
                        )
                      }}
                    </span>
                  </a>
                </div>
                <div class="content">
                  <div>
                    <div class="location label">
                      <span>{{ $t("transfer.from") }}</span>
                      <i class="material-icons">home</i>
                    </div>
                    <div class="location">
                      <span class="server">
                        {{ transfer.agent.localAddress }}</span
                      ><i class="material-icons">chevron_right</i>
                      <span class="path">{{ transfer.items[0].from }}</span>
                    </div>
                  </div>
                  <div>
                    <div class="location label">
                      <span>{{ $t("transfer.to") }}</span>
                      <i class="material-icons">vpn_lock</i>
                    </div>
                    <div class="location">
                      <span class="server">
                        {{ transfer.agent.host }}:{{
                          transfer.agent.port
                        }}</span
                      ><i class="material-icons">chevron_right</i>
                      <span class="path">{{ transfer.items[0].to }}</span>
                    </div>
                  </div>
                </div>
              </div>
              <div class="content">
                <div v-if="transfer.showItems" class="show-paths">
                  <a href="#" @click.prevent="showFullPaths(transfer)">
                    <span v-if="transfer.showPaths === false">
                      {{ $t("transfer.showPaths") }}
                    </span>
                    <span v-else>
                      {{ $t("transfer.hidePaths") }}
                    </span>
                  </a>
                </div>
                <ul v-if="transfer.showItems">
                  <li
                    v-for="(item, index) in transfer.items"
                    :key="index"
                    :class="{
                      item: true,
                      compact: !transfer.showPaths,
                    }"
                  >
                    <span class="path">{{ item.from }}</span>
                    <span class="name">{{ item.name }}</span>
                    <i class="material-icons">east</i>
                    <span class="to">{{ item.to }}</span>
                  </li>
                </ul>
              </div>
            </div>
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
    numFiles: function (transfer) {
      if (transfer.numDirs === undefined) {
        transfer.numDirs = transfer.items.reduce(function (total, item) {
          return total + (item.isDir ? 0 : 1);
        }, 0);
      }
      return transfer.numDirs;
    },
    numDirs: function (transfer) {
      if (transfer.numFiles === undefined) {
        transfer.numFiles = transfer.items.reduce(function (total, item) {
          return total + (item.isDir ? 1 : 0);
        }, 0);
      }
      return transfer.numFiles;
    },
    handleCancel: function (transfer) {
      let transferID = transfer.transferID;

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
    showItems: function (transfer) {
      transfers.update(this.$store, {
        transferID: transfer.transferID,
        showItems: !transfer.showItems,
      });
    },
    showFullPaths: function (transfer) {
      transfers.update(this.$store, {
        transferID: transfer.transferID,
        showPaths: !transfer.showPaths,
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
  max-height: 88%;
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
  font-size: small;
  margin: 0.25em 0 0.75em 0;
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

.transfer
  > .content
  > .icon:not(.icon-error):not(.icon-success):not(.icon-canceled)
  i {
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

.transfer .icon-error i {
  color: var(--dark-red);
}

.transfer .icon-success i {
  color: var(--icon-blue);
}

.transfer .icon-canceled i {
  color: var(--mid-grey);
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

.transfer > .content > .details .content .show-paths {
  margin: 0.5em 0 0 0;
  text-align: right;
  text-decoration: underline;
  font-size: small;
}

.transfer > .content > .details .content .item .path {
  color: var(--mid-grey);
}

.transfer > .content > .details .content .item.compact .path {
  display: none;
}

.transfer > .content > .details .content .item.compact .name {
  font-weight: normal;
}

.transfer > .content > .details .content .item.compact i {
  display: none;
}

.transfer > .content > .details .content .item.compact .to {
  display: none;
}

.transfer > .content > .details .content .item .name {
  font-weight: bold;
  margin-right: 0.25em;
}

.transfer > .content > .details .content .to {
  margin-left: 1.25em;
}

.transfer > .content > .details > div > .content {
  clear: left;
}

.transfer > .content > .details > div > .content i {
  position: absolute;
  font-size: 1rem;
  line-height: 1.25rem;
}

.transfer > .content > .details > div > .content ul {
  margin: 0.3em 0 0 0;
  padding: 0 0 0 1em;
}

.transfer > .content > .details > div > .content ul > li {
  line-height: 1.33em;
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

.transfer > .content > .details > .icon i {
  font-size: 1em;
  vertical-align: middle;
}

.transfer > .content > .details > .icon .label {
  font-size: 90%;
  margin: 0 0 0 0.25em;
}

.transfer > .content > .details > div > .summary {
  margin: 1.5em 0 0 0;
}

.transfer > .content > .details > div > .summary > div > a {
  text-decoration: underline;
}

.transfer > .content > .details > div > .summary .content i {
  font-size: 1.25em;
  margin: 0 0.33em;
}

.transfer > .content > .details > div > .summary > .content {
  margin: 0.5em 0 0 0;
  border: 1px solid var(--card-border);
  border-radius: 2px;
  padding: 0.5em 0 0.25em 0;
  background: var(--distinct-background2);
}

.transfer > .content > .details .summary > .content .location.label {
  float: left;
  clear: left;
  width: 5em;
  text-align: right;
  font-style: italic;
  opacity: 0.9;
}

.transfer .summary .location:not(:first-child).label,
.transfer .summary .location:not(:first-child) {
  margin-top: 0.25em;
}

.transfer > .content > .details .summary > .content .location:not(.label) i {
  margin: 0;
  color: var(--card-border);
}

.transfer > .content > .details .summary > .content .location {
  line-height: 1.5em;
  display: table;
}

.transfer > .content > .details .summary > .content .server {
  font-weight: bold;
}

.transfer > .content > .details .summary > .content .path {
  color: var(--mid-grey);
}

.transfer > .content > .details .summary > .content span {
  position: relative;
  bottom: 0.3em;
}

.transfer > .content > .details > div > .summary > div.title {
  font-size: initial;
  padding: 0.6em 0;
}

@media (max-width: 736px) {
}

section.card-action {
  padding: 0.5em;
}
</style>
