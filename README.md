# WebSCP

[![Go Report Card](https://goreportcard.com/badge/github.com/marekful/webscp?style=flat-square)](https://goreportcard.com/report/github.com/filebrowser/filebrowser) [![Version](https://img.shields.io/github/v/release/marekful/webscp?display_name=tag&include_prereleases)](https://github.com/marekful/webscp/releases/latest) [![Docker](https://img.shields.io/docker/v/marekful/webscp-files/rc-beta-3?label=files)](https://hub.docker.com/r/marekful/webscp-files/tags) [![Docker](https://img.shields.io/docker/v/marekful/webscp-agent/rc-beta-3-alpine?label=agent)](https://hub.docker.com/r/marekful/webscp-agent/tags) [![Docker](https://img.shields.io/docker/v/marekful/webscp-agent/rc-beta-3-debian?label=agent)](https://hub.docker.com/r/marekful/webscp-agent/tags)

WebSCP provides a simple, graphical frontend for securely copying files between Linux servers using [scp](https://linux.die.net/man/1/scp).

It builds on the brilliant web browser based file manager, [File Browser](https://github.com/filebrowser/filebrowser), and extends it with the following features:

* Single Sign On using Open ID Connect Discovery
* 2 click setup to connect to other WebSCP instances
* A graphical frontend for securely copying files between servers using [scp](https://linux.die.net/man/1/scp)

The file manager comes with a configurable root directory and it can be used to upload, delete, preview, rename and edit your files. It allows the creation of multiple users and each user can have its own root directory and permissions.

## Features

For File Browser features concering local operations, please refer to the original docs at [https://filebrowser.org/features](https://filebrowser.org/features).

* Seamleassly set up SSH key authentication based connectoins using a one time access token from the remote
* Connections are only visible to the user who created them
* Connection are tied to the remote user who generated the access token
* Authentication with the remote user's credentials is required when a connection is beeing used
* Upload files and/or directories to remote servers using existing connections
* Option to compress files before upload
* User permissions and restrictions are respected on both on the local and remote sides
* Detect file name conflicts before remote copy
* Option to keep or replace existing files on the remote in case of conflict
* Keep track of remote operations or cancel them through the UI

## Installation

WebSCP requires Docker to run. See sample configuration and installation instructions on [GitHub](https://github.com/marekful/webscp/tree/master/agent/install).
