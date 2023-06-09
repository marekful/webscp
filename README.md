# WebSCP

[![CodeQL](https://github.com/marekful/webscp/actions/workflows/codeql.yml/badge.svg?branch=main)](https://github.com/marekful/webscp/actions/workflows/codeql.yml) [![OSSF Scorecard](https://img.shields.io/ossf-scorecard/github.com/marekful/webscp?label=ossf%20score)](https://img.shields.io/ossf-scorecard/github.com/marekful/webscp) [![Go Report Card](https://goreportcard.com/badge/github.com/marekful/webscp?style=flat-square)](https://goreportcard.com/report/github.com/marekful/webscp/backend) [![Version](https://img.shields.io/github/v/release/marekful/webscp?display_name=tag&include_prereleases)](https://github.com/marekful/webscp/releases/latest) [![Docker](https://img.shields.io/docker/v/marekful/webscp-files/latest?label=files&logo=docker&color=blue)](https://hub.docker.com/r/marekful/webscp-files/tags) [![Docker](https://img.shields.io/docker/v/marekful/webscp-agent/latest?label=agent&logo=docker&color=blue)](https://hub.docker.com/r/marekful/webscp-agent/tags) 

WebSCP provides a simple, graphical frontend for securely copying files between your Linux or MacOS machines. 

https://user-images.githubusercontent.com/10281476/234520425-08791a15-6bf4-4395-980c-96cb6c79f7b3.mov

It builds on the brilliant web browser based file manager, [File Browser](https://github.com/filebrowser/filebrowser), and extends it with the following features:

* Single Sign On using Open ID Connect Discovery
* 2 click setup to connect to other WebSCP instances
* A graphical frontend for securely copying files between servers

It uses [OpenSSH](https://en.wikipedia.org/wiki/OpenSSH) to access remote machines and [scp](https://linux.die.net/man/1/scp) to copy files.

https://user-images.githubusercontent.com/10281476/234520613-ee4319ef-dec7-407e-a064-857b8a9c3faf.mov

https://user-images.githubusercontent.com/10281476/234520712-72b5a7e5-ce96-4ce7-b1b0-3387027f2edb.mov

The file manager comes with a configurable root directory and it can be used to upload, delete, preview, rename and edit your files. It allows the creation of multiple users and each user can have its own root directory and permissions.

## Features

For File Browser features concering local operations, please refer to the original [documentation](https://filebrowser.org/features).

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

## Compatibility & Installation

WebSCP is compatible with newer (64bit) MacOS (Intel, M1, M2) and Linux (Intel, ARM) and it requires Docker.

Please read the [Installation and Version Upgrade Guide](https://github.com/marekful/webscp/tree/main/install) for details.

<div align='right'>
  <a href="https://bestpractices.coreinfrastructure.org/projects/7344"><img src="https://bestpractices.coreinfrastructure.org/projects/7344/badge"></a>
</div>
