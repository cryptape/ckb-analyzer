## Network

* Public Nodes
  开放连接的节点数。多个地区的探测器尝试连接所有可达的节点。节点以 peer_id 作为唯一标识，与 ip 类型无关，所以共用同一个 peer_id 的节点将被统计为一个。

* Distribution of Node Version
  客户端版本的分布。收集探测器和 bootnodes 的所有对端信息，汇总出节点的客户端版本的分布。

* Connection Duration Distribution
  收集与探测器连接的节点中，保持连接的时长在一分钟、一小时、一天、一个月的节点数。

* Propagation Percentile of Transactions and Blocks
  探测器连接所有可达的节点，收集对端发送的 TransactionHashes、Transactions、CompactBlock，统计出相同消息传播到 50%/80%/90%/95% 的网络节点的时长（以第一次接受到相应消息的时间戳为起始）。

## Transaction

* Traffic of Pending Transactions
  交易池的吞吐量，以交易数为计量单位。

* Traffic of Removed Transactions
  交易池移除的交易，以交易数为计量单位。交易池之所以移除某些交易，是因为部分交易已经不合法，比如与链上交易冲突。

* Delay of Transaction from Being Entering to Committed
  交易从进入交易池到变成 committed 状态的延迟，以时间为计量单位。

* Delay of Transaction from Being Proposed to Committed (2 Phase Commitment)
  交易从 proposed 状态变成 committed 状态的延迟，以块数为计量单位。通过比较交易第一次被 propose 的高度与被 commit 的高度，算出两阶段提交的延迟。在网络通畅的情况下，延迟应该都是 2，若有延迟超过 2 的交易出现，可能表示整体网络不太通畅。

## Activity on the Network
