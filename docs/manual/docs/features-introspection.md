---
id: features-introspection
title: Introspection
sidebar_label: Introspection
---

In an ideal world software, once installed and configured, runs perfectly without ever needing attention.
Of course reality tells a different story.

Software bugs and evolution, transient network issues, miss-configurations ...
Many things can go wrong and the symptoms are often unclear.
Distributed systems also mean that errors often propagate across processes
and servers so the error location is far from the error origin.

On top of all that, distributed systems are complex to follow and
simple questions about correct functioning become hard to answer.

Replicante is a distributed system and as such it is subject to all the above complications.
To help users and administrators understand and manage installations, as well as troubleshoot issues,
Replicante provides a set of features to introspect the system and trace its activity.


## Events trail
Replicante is an event-driven system at its core.
Because of that, most activities of the system can be explained and monitored by looking at events.

The [events](features-events.md) section explains how to view and programmatically follow events.


## Metrics
Information about internal operation of replicante is exposed through metrics.
These can be used to monitor the health and activity of a process as well as its performance.

Metrics are exposed in [Prometheus](https://prometheus.io/) format by the API endpoint `/api/v1/metrics`.


## Logging
Logging is a good way to see exactly what one system was doing at a precise point in time.
Replicante provides structured logging so administrators can see what is happening and in what context.

By itself this is needed but not that great.
The real power of structured logging comes in with centralised log collection:
the logs from every server are collected and indexed in a central location along with other services.


### Configuration
Various logging backends are supported so that replicante can fit into your infrastructure
and some options are provided to user regardless of the backend of choice.
All options are under the `logging` section.
The details are documented in the [configuration reference](admin-config.md).

Below are the supported backends:

  * `json` (default) outputs logs to standard output in JSON format:
    * The output is not the easiest to read directly.
    * It works well with process supervisors that expect logs from standard output (i.e, docker).
    * Lines can be processed by any tools that understand JSON (i.e, `jq` or crafted scripts).
  * `journald` sends logs to journald directly (systemd's logging facility):
    * `journald` is available only if enabled at compile time.
    * `journald` is requires a server running systemd.


## Distributed Tracing
Following the details of an operation from start to finish when it spans several servers
can be a challenge.
Thankfully there are tools to address this challenge: distributed tracers.

Distributed tracers are central systems that collect segments of operations from different servers
and combine them together to show the entire story of a full operation.


### Configuration
Replicante supports integration with some distributed tracing tools compatible with the
[OpenTracing](http://opentracing.io/) specification.

By default distributed tracing is disabled but it can be [configured](admin-config.md)
with the options under the `tracing` section.