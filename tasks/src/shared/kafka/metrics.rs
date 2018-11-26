use prometheus::GaugeVec;
use prometheus::Opts;
use prometheus::Registry;

use rdkafka::ClientContext;
use rdkafka::consumer::ConsumerContext;
use rdkafka::statistics::Statistics;
use slog::Logger;


lazy_static! {
    pub static ref KAFKA_BROKER_OUTBUF_CNT: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_broker_outbuf_cnt",
            "Number of requests awaiting transmission to broker"
        ),
        &["role", "broker"]
    ).expect("Failed to create KAFKA_BROKER_OUTBUF_CNT gauge");

    pub static ref KAFKA_BROKER_OUTBUF_MSG_CNT: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_broker_outbuf_msg_cnt",
            "Number of messages awaiting transmission to broker"
        ),
        &["role", "broker"]
    ).expect("Failed to create KAFKA_BROKER_OUTBUF_MSG_CNT gauge");

    pub static ref KAFKA_BROKER_REQ_TIMEOUTS: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_broker_req_timeouts",
            "Total number of requests timed out"
        ),
        &["role", "broker"]
    ).expect("Failed to create KAFKA_BROKER_REQ_TIMEOUTS gauge");

    pub static ref KAFKA_BROKER_RX: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_broker_rx",
            "Total number of responses received"
        ),
        &["role", "broker"]
    ).expect("Failed to create KAFKA_BROKER_RX gauge");

    pub static ref KAFKA_BROKER_RXBYTES: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_broker_rxbytes",
            "Total number of bytes received"
        ),
        &["role", "broker"]
    ).expect("Failed to create KAFKA_BROKER_RXBYTES gauge");

    pub static ref KAFKA_BROKER_RXCORRIDERRS: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_broker_rxcorriderrs",
            "Total number of unmatched correlation ids in response"
        ),
        &["role", "broker"]
    ).expect("Failed to create KAFKA_BROKER_RXCORRIDERRS gauge");

    pub static ref KAFKA_BROKER_RXERRS: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_broker_rxerrs",
            "Total number of receive errors"
        ),
        &["role", "broker"]
    ).expect("Failed to create KAFKA_BROKER_RXERRS gauge");

    pub static ref KAFKA_BROKER_RXPARTIAL: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_broker_rxpartial",
            "Total number of partial MessageSets received"
        ),
        &["role", "broker"]
    ).expect("Failed to create KAFKA_BROKER_RXPARTIAL gauge");

    pub static ref KAFKA_BROKER_TX: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_broker_tx",
            "Total number of requests sent"
        ),
        &["role", "broker"]
    ).expect("Failed to create KAFKA_BROKER_TX gauge");

    pub static ref KAFKA_BROKER_TXBYTES: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_broker_txbytes",
            "Total number of bytes sent"
        ),
        &["role", "broker"]
    ).expect("Failed to create KAFKA_BROKER_TXBYTES gauge");

    pub static ref KAFKA_BROKER_TXERRS: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_broker_txerrs",
            "Total number of transmission errors"
        ),
        &["role", "broker"]
    ).expect("Failed to create KAFKA_BROKER_TXERRS gauge");

    pub static ref KAFKA_BROKER_TXRETRIES: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_broker_txretries",
            "Total number of request retries"
        ),
        &["role", "broker"]
    ).expect("Failed to create KAFKA_BROKER_TXRETRIES gauge");

    pub static ref KAFKA_BROKER_WAITRESP_CNT: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_broker_waitresp_cnt",
            "Number of requests in-flight to broker awaiting response"
        ),
        &["role", "broker"]
    ).expect("Failed to create KAFKA_BROKER_WAITRESP_CNT gauge");

    pub static ref KAFKA_BROKER_WAITRESP_MSG_CNT: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_broker_waitresp_msg_cnt",
            "Number of messages in-flight to broker awaitign response"
        ),
        &["role", "broker"]
    ).expect("Failed to create KAFKA_BROKER_WAITRESP_MSG_CNT gauge");

    pub static ref KAFKA_BROKER_WAKEUPS: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_broker_wakeups",
            "Broker thread poll wakeups"
        ),
        &["role", "broker"]
    ).expect("Failed to create KAFKA_BROKER_WAKEUPS gauge");

    pub static ref KAFKA_BROKER_ZBUF_GROW: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_broker_zbuf_grow",
            "Total number of decompression buffer size increases"
        ),
        &["role", "broker"]
    ).expect("Failed to create KAFKA_BROKER_ZBUF_GROW gauge");

    pub static ref KAFKA_CGRP_ASSIGNMENT_SIZE: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_cgrp_assignment_size",
            "Current assignment's partition count"
        ),
        &["role"]
    ).expect("Failed to create KAFKA_CGRP_ASSIGNMENT_SIZE gauge");

    pub static ref KAFKA_MSG_CNT: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_msg_cnt",
            "Current number of messages in producer queues"
        ),
        &["role"]
    ).expect("Failed to create KAFKA_MSG_CNT gauge");

    pub static ref KAFKA_PARTITION_COMMITTED_OFFSET: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_partition_committed_offset",
            "Last committed offset"
        ),
        &["role", "topic", "partition"]
    ).expect("Failed to create KAFKA_PARTITION_COMMITTED_OFFSET gauge");

    pub static ref KAFKA_PARTITION_CONSUMER_LAG: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_partition_consumer_lag",
            "Difference between hi_offset - max(app_offset, committed_offset)"
        ),
        &["role", "topic", "partition"]
    ).expect("Failed to create KAFKA_PARTITION_CONSUMER_LAG gauge");

    pub static ref KAFKA_PARTITION_FETCHQ_CNT: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_partition_fetchq_cnt",
            "Number of pre-fetched messages in fetch queue"
        ),
        &["role", "topic", "partition"]
    ).expect("Failed to create KAFKA_PARTITION_FETCHQ_CNT gauge");

    pub static ref KAFKA_PARTITION_FETCHQ_SIZE: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_partition_fetchq_size",
            "Bytes in fetchq"
        ),
        &["role", "topic", "partition"]
    ).expect("Failed to create KAFKA_PARTITION_FETCHQ_SIZE gauge");

    pub static ref KAFKA_PARTITION_MSGQ_BYTES: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_partition_msgq_bytes",
            "Number of bytes in msgq_cnt"
        ),
        &["role", "topic", "partition"]
    ).expect("Failed to create KAFKA_PARTITION_MSGQ_BYTES gauge");

    pub static ref KAFKA_PARTITION_MSGQ_CNT: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_partition_msgq_cnt",
            "Number of messages waiting to be produced in first-level queue"
        ),
        &["role", "topic", "partition"]
    ).expect("Failed to create KAFKA_PARTITION_MSGQ_CNT gauge");

    pub static ref KAFKA_PARTITION_TXBYTES: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_partition_txbytes",
            "Total number of bytes transmitted for txmsgs"
        ),
        &["role", "topic", "partition"]
    ).expect("Failed to create KAFKA_PARTITION_TXBYTES gauge");

    pub static ref KAFKA_PARTITION_TXMSGS: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_partition_txmsgs",
            "Total number of messages transmitted (produced)"
        ),
        &["role", "topic", "partition"]
    ).expect("Failed to create KAFKA_PARTITION_TXMSGS gauge");

    pub static ref KAFKA_PARTITION_XMIT_MSGQ_BYTES: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_partition_xmit_msgq_bytes",
            "Number of bytes in xmit_msgq"
        ),
        &["role", "topic", "partition"]
    ).expect("Failed to create KAFKA_PARTITION_XMIT_MSGQ_BYTES gauge");

    pub static ref KAFKA_PARTITION_XMIT_MSGQ_CNT: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_partition_xmit_msgq_cnt",
            "Number of messages ready to be produced in transmit queue"
        ),
        &["role", "topic", "partition"]
    ).expect("Failed to create KAFKA_PARTITION_XMIT_MSGQ_CNT gauge");

    pub static ref KAFKA_REPLYQ: GaugeVec = GaugeVec::new(
        Opts::new(
            "replicore_tasks_kafka_replyq",
            "Number of ops waiting in queue for application to serve with rd_kafka_poll()"
        ),
        &["role"]
    ).expect("Failed to create KAFKA_REPLYQ_GAUGE gauge");
}


/// Attemps to register metrics with the Registry.
///
/// Metrics that fail to register are logged and ignored.
pub fn register_metrics(logger: &Logger, registry: &Registry) {
    if let Err(err) = registry.register(Box::new(KAFKA_BROKER_OUTBUF_CNT.clone())) {
        debug!(logger, "Failed to register KAFKA_BROKER_OUTBUF_CNT"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_BROKER_OUTBUF_MSG_CNT.clone())) {
        debug!(logger, "Failed to register KAFKA_BROKER_OUTBUF_MSG_CNT"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_BROKER_REQ_TIMEOUTS.clone())) {
        debug!(logger, "Failed to register KAFKA_BROKER_REQ_TIMEOUTS"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_BROKER_RX.clone())) {
        debug!(logger, "Failed to register KAFKA_BROKER_RX"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_BROKER_RXBYTES.clone())) {
        debug!(logger, "Failed to register KAFKA_BROKER_RXBYTES"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_BROKER_RXCORRIDERRS.clone())) {
        debug!(logger, "Failed to register KAFKA_BROKER_RXCORRIDERRS"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_BROKER_RXERRS.clone())) {
        debug!(logger, "Failed to register KAFKA_BROKER_RXERRS"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_BROKER_RXPARTIAL.clone())) {
        debug!(logger, "Failed to register KAFKA_BROKER_RXPARTIAL"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_BROKER_TX.clone())) {
        debug!(logger, "Failed to register KAFKA_BROKER_TX"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_BROKER_TXBYTES.clone())) {
        debug!(logger, "Failed to register KAFKA_BROKER_TXBYTES"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_BROKER_TXERRS.clone())) {
        debug!(logger, "Failed to register KAFKA_BROKER_TXERRS"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_BROKER_TXRETRIES.clone())) {
        debug!(logger, "Failed to register KAFKA_BROKER_TXRETRIES"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_BROKER_WAITRESP_CNT.clone())) {
        debug!(logger, "Failed to register KAFKA_BROKER_WAITRESP_CNT"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_BROKER_WAITRESP_MSG_CNT.clone())) {
        debug!(logger, "Failed to register KAFKA_BROKER_WAITRESP_MSG_CNT"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_BROKER_WAKEUPS.clone())) {
        debug!(logger, "Failed to register KAFKA_BROKER_WAKEUPS"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_BROKER_ZBUF_GROW.clone())) {
        debug!(logger, "Failed to register KAFKA_BROKER_ZBUF_GROW"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_MSG_CNT.clone())) {
        debug!(logger, "Failed to register KAFKA_MSG_CNT"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_PARTITION_COMMITTED_OFFSET.clone())) {
        debug!(logger, "Failed to register KAFKA_PARTITION_COMMITTED_OFFSET"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_PARTITION_CONSUMER_LAG.clone())) {
        debug!(logger, "Failed to register KAFKA_PARTITION_CONSUMER_LAG"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_PARTITION_FETCHQ_CNT.clone())) {
        debug!(logger, "Failed to register KAFKA_PARTITION_FETCHQ_CNT"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_PARTITION_FETCHQ_SIZE.clone())) {
        debug!(logger, "Failed to register KAFKA_PARTITION_FETCHQ_SIZE"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_PARTITION_MSGQ_BYTES.clone())) {
        debug!(logger, "Failed to register KAFKA_PARTITION_MSGQ_BYTES"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_PARTITION_MSGQ_CNT.clone())) {
        debug!(logger, "Failed to register KAFKA_PARTITION_MSGQ_CNT"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_PARTITION_TXBYTES.clone())) {
        debug!(logger, "Failed to register KAFKA_PARTITION_TXBYTES"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_PARTITION_TXMSGS.clone())) {
        debug!(logger, "Failed to register KAFKA_PARTITION_TXMSGS"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_PARTITION_XMIT_MSGQ_BYTES.clone())) {
        debug!(logger, "Failed to register KAFKA_PARTITION_XMIT_MSGQ_BYTES"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_PARTITION_XMIT_MSGQ_CNT.clone())) {
        debug!(logger, "Failed to register KAFKA_PARTITION_XMIT_MSGQ_CNT"; "error" => ?err);
    }
    if let Err(err) = registry.register(Box::new(KAFKA_REPLYQ.clone())) {
        debug!(logger, "Failed to register KAFKA_REPLYQ"; "error" => ?err);
    }
}


/// A kafka client context to expose stats over prometheus.
pub struct ClientStatsContext {
    role: String,
}

impl ClientStatsContext {
    pub fn new<S: Into<String>>(role: S) -> ClientStatsContext {
        let role = role.into();
        ClientStatsContext { role, }
    }
}

impl ClientContext for ClientStatsContext {
    fn stats(&self, stats: Statistics) {
        KAFKA_MSG_CNT.with_label_values(&[&self.role]).set(stats.msg_cnt as f64);
        KAFKA_REPLYQ.with_label_values(&[&self.role]).set(stats.replyq as f64);
        for broker in stats.brokers.values() {
            KAFKA_BROKER_OUTBUF_CNT.with_label_values(&[&self.role, &broker.name])
                .set(broker.outbuf_cnt as f64);
            KAFKA_BROKER_OUTBUF_MSG_CNT.with_label_values(&[&self.role, &broker.name])
                .set(broker.outbuf_msg_cnt as f64);
            KAFKA_BROKER_REQ_TIMEOUTS.with_label_values(&[&self.role, &broker.name])
                .set(broker.req_timeouts as f64);
            KAFKA_BROKER_RX.with_label_values(&[&self.role, &broker.name])
                .set(broker.rx as f64);
            KAFKA_BROKER_RXBYTES.with_label_values(&[&self.role, &broker.name])
                .set(broker.rxbytes as f64);
            KAFKA_BROKER_RXCORRIDERRS.with_label_values(&[&self.role, &broker.name])
                .set(broker.rxcorriderrs as f64);
            KAFKA_BROKER_RXERRS.with_label_values(&[&self.role, &broker.name])
                .set(broker.rxerrs as f64);
            KAFKA_BROKER_RXPARTIAL.with_label_values(&[&self.role, &broker.name])
                .set(broker.rxpartial as f64);
            KAFKA_BROKER_TX.with_label_values(&[&self.role, &broker.name])
                .set(broker.tx as f64);
            KAFKA_BROKER_TXBYTES.with_label_values(&[&self.role, &broker.name])
                .set(broker.txbytes as f64);
            KAFKA_BROKER_TXERRS.with_label_values(&[&self.role, &broker.name])
                .set(broker.txerrs as f64);
            KAFKA_BROKER_TXRETRIES.with_label_values(&[&self.role, &broker.name])
                .set(broker.txretries as f64);
            KAFKA_BROKER_WAITRESP_CNT.with_label_values(&[&self.role, &broker.name])
                .set(broker.waitresp_cnt as f64);
            KAFKA_BROKER_WAITRESP_MSG_CNT.with_label_values(&[&self.role, &broker.name])
                .set(broker.waitresp_msg_cnt as f64);
            KAFKA_BROKER_WAKEUPS.with_label_values(&[&self.role, &broker.name])
                .set(broker.wakeups.unwrap_or(0) as f64);
            KAFKA_BROKER_ZBUF_GROW.with_label_values(&[&self.role, &broker.name])
                .set(broker.zbuf_grow as f64);
        }
        for topic in stats.topics.values() {
            for partition in topic.partitions.values() {
                let id = partition.partition.to_string();
                KAFKA_PARTITION_COMMITTED_OFFSET.with_label_values(&[&self.role, &topic.topic, &id])
                    .set(partition.committed_offset as f64);
                KAFKA_PARTITION_CONSUMER_LAG.with_label_values(&[&self.role, &topic.topic, &id])
                    .set(partition.consumer_lag as f64);
                KAFKA_PARTITION_FETCHQ_CNT.with_label_values(&[&self.role, &topic.topic, &id])
                    .set(partition.fetchq_cnt as f64);
                KAFKA_PARTITION_FETCHQ_SIZE.with_label_values(&[&self.role, &topic.topic, &id])
                    .set(partition.fetchq_size as f64);
                KAFKA_PARTITION_MSGQ_BYTES.with_label_values(&[&self.role, &topic.topic, &id])
                    .set(partition.msgq_bytes as f64);
                KAFKA_PARTITION_MSGQ_CNT.with_label_values(&[&self.role, &topic.topic, &id])
                    .set(partition.msgq_cnt as f64);
                KAFKA_PARTITION_TXBYTES.with_label_values(&[&self.role, &topic.topic, &id])
                    .set(partition.txbytes as f64);
                KAFKA_PARTITION_TXMSGS.with_label_values(&[&self.role, &topic.topic, &id])
                    .set(partition.txmsgs as f64);
                KAFKA_PARTITION_XMIT_MSGQ_BYTES.with_label_values(&[&self.role, &topic.topic, &id])
                    .set(partition.xmit_msgq_bytes as f64);
                KAFKA_PARTITION_XMIT_MSGQ_CNT.with_label_values(&[&self.role, &topic.topic, &id])
                    .set(partition.xmit_msgq_cnt as f64);
            }
        }
        if let Some(cgrp) = stats.cgrp {
            KAFKA_CGRP_ASSIGNMENT_SIZE.with_label_values(&[&self.role])
                .set(cgrp.assignment_size as f64);
        }
    }
}

impl ConsumerContext for ClientStatsContext {}