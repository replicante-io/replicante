---
id: admin-config
title: Configuration
sidebar_label: Configuration
---

Replicante provides a large set of configuration options with reasonable defaults.
This supports common use cases where only a handful of options need attention
as well as advanced setup where the user has the power to fine tune most details.


## Required configuration
Some options do not have reasonable defaults so users will have to set them explicitly:

  * [Agents discovery](features-discovery.md) (vastly depends on user needs).
  * Storage configuration (specifically: address of the DB server).


## All configuration options
All options are documented in the
[example configuration file](https://github.com/replicante-io/replicante/blob/master/replicante.example.yaml)
at the root of the repo, also shown below.

This file shows all options with their defaults and explains their meaning and available settings.
As mentioned above, common use cases should be able to ignore most options if users are so inclined.

Details of these options are documented in the features they influence.

```yaml
# The section below is for the API interface configuration.
api:
  # The network interface and port to bind the API server onto.
  #
  # By default, only bind to the loopback interface.
  # Production environments should place an HTTPS proxy in front of the API.
  bind: '127.0.0.1:16016'


# The section below is for agent discovery configuration.
#
# Discovery is the way the agents that should be managed are found.
# Replicante core then interact with agents by initiating connections out to them.
discovery:
  # Interval (in seconds) to wait between agent discovery runs.
  interval: 60

  # Discovery backends configuration.
  #
  # Backends are a way to support an extensible set of discovery systems.
  # Each backend has its own options as described below.
  #
  # Available backends are:
  #
  #   * `files`: discover agents from local config files.
  backends:
    # The `files` backend discover agents from files.
    #
    # The `files` backend can be useful to delegate discovery to unsupported systems.
    # Examples are configuration managemnet tools (ansible, chef, puppet, ...).
    #
    # This is a list of files that are periodically read to perform discovery.
    files: []


# The section below is for logging configuration.
logging:
  # Flush logs asynchronously.
  #
  # Pro:
  #     Async log flushing is more efficient as processes
  #     are not blocked waiting for logging backends to complete.
  #
  # Con:
  #     If the process crashes logs in the buffer may be lost.
  #
  # Recommendation:
  #     Keep async logging enabled unless replicante is crashing
  #     and the logs don't have any indications of why.
  #
  #     Async logging may also be disabled in testing, debugging,
  #     or developing environments.
  async: true

  # Logging backend configuration.
  backend:
    # The backend to send logs to.
    # This option also determines the format and destination of logs.
    #
    # Available options:
    #
    #   * 'json': prints JSON formatted logs to standard output.
    #   * 'journald': sends logs to systemd journal (if enabled at compile time).
    name: json

    # Any backend-specific option is set here.
    # The available options vary from backend to backend and are documented below.
    #
    # *** None available at this time ***
    #options:

  # The minimum logging level.
  #
  # Available options:
  #
  #   * 'critical'
  #   * 'error'
  #   * 'warning'
  #   * 'info'
  #   * 'debug' (only available in debug builds)
  level: info


# The section below is for storage configuration.
storage:
  # The database to use for persistent storage.
  #
  # !!! DO NOT CHANGE AFTER INITIAL CONFIGURATION !!!
  # This option is to allow users to choose a supported database that best fits
  # their use and environment.
  #
  # To change this option you will need to "Update by rebuild".
  # See the documentation for more details on this process.
  #
  # Available options:
  #
  #   * 'mongodb' (recommended)
  backend: mongodb

  # Any backend-specific option is set here.
  # The available options vary from backend to backend and are documented below.
  #
  # MongoDB options
  options:
    # Name of the MongoDB database to use for persistence.
    #
    # !!! DO NOT CHANGE AFTER INITIAL CONFIGURATION !!!
    # This option is to allow users to choose a supported database that best fits
    # their use and environment.
    #
    # To change this option you will need to "Update by rebuild".
    # See the documentation for more details on this process.
    db: replicante  # (recommended)

    # URI of the MongoDB Replica Set or sharded cluster to connect to.
    uri: mongodb://localhost:27017/


# The section below is for distributed tracing configuration.
tracing:
  # The distributed tracing backend to integrate with.
  #
  # Available options:
  #
  #   * 'noop'
  #   * 'zipkin'
  backend: noop

  # Any backend-specific option is set here.
  # The available options vary from tracer to tracer and are documented below.
  #
  # Zipkin options
  #options:
  #  # (required) The service name for this zipkin endpoint.
  #  service_name: replicante
  #
  #  # (required) List of kafka seed hostnames.
  #  kafka:
  #    - HOST1:9092
  #    - HOST2:9092
  #
  #  # The kafka topic to publish spans to.
  #  topic: zipkin
```