# WebSCP

[![Go Report Card](https://goreportcard.com/badge/github.com/marekful/webscp?style=flat-square)](https://goreportcard.com/report/github.com/filebrowser/filebrowser) [![Version](https://img.shields.io/github/v/release/marekful/webscp?display_name=tag&include_prereleases)](https://github.com/marekful/webscp/releases/latest) [![Docker](https://img.shields.io/docker/v/marekful/webscp-files/rc-beta-4.4?label=files&logo=docker)](https://hub.docker.com/r/marekful/webscp-files/tags) [![Docker](https://img.shields.io/docker/v/marekful/webscp-agent/rc-beta-4.4-alpine?label=agent&logo=docker)](https://hub.docker.com/r/marekful/webscp-agent/tags) [![Docker](https://img.shields.io/docker/v/marekful/webscp-agent/rc-beta-4.4-debian?label=agent&logo=docker)](https://hub.docker.com/r/marekful/webscp-agent/tags)

WebSCP provides a simple, graphical frontend for securely copying files between your Linux or MacOS machines. It uses [OpenSSH](https://en.wikipedia.org/wiki/OpenSSH) to access remote machines and [scp](https://linux.die.net/man/1/scp) to copy files.

https://user-images.githubusercontent.com/10281476/234520425-08791a15-6bf4-4395-980c-96cb6c79f7b3.mov

It builds on the brilliant web browser based file manager, [File Browser](https://github.com/filebrowser/filebrowser), and extends it with the following features:

* Single Sign On using Open ID Connect Discovery
* 2 click setup to connect to other WebSCP instances
* A graphical frontend for securely copying files between servers using [scp](https://linux.die.net/man/1/scp)

https://user-images.githubusercontent.com/10281476/234520613-ee4319ef-dec7-407e-a064-857b8a9c3faf.mov

https://user-images.githubusercontent.com/10281476/234520712-72b5a7e5-ce96-4ce7-b1b0-3387027f2edb.mov

The file manager comes with a configurable root directory and it can be used to upload, delete, preview, rename and edit your files. It allows the creation of multiple users and each user can have its own root directory and permissions.

## Features

For File Browser features concering local operations, please refer to the original docs at [https://filebrowser.org/features](https://filebrowser.org/features).

* Seamleassly set up SSH key authentication based connectoins using a one time access token from the remote
* Connections are only visible to the user who created them
* Connection are tied to the remote user who generated the access token
* Authentication with the remote user's credentials is required when a connection is beeing used
* Upload files and/or directories to remote servers using existing connections
* Option to compress files before upload
* User permissions and restrictions are respected on both the local and remote sides
* Detect file name conflicts before remote copy
* Option to keep or replace existing files on the remote in case of conflict
* Keep track of remote operations or cancel them through the UI

## Installation

WebSCP requires Docker to run. See sample configuration and installation instructions on [GitHub](https://github.com/marekful/webscp/tree/main/agent/install).
