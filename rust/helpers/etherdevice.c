#include "linux/netdevice.h"
#include <linux/etherdevice.h>
#include <linux/skbuff.h>

void *rust_helper_netdev_priv(const struct net_device *dev)
{
	return netdev_priv(dev);
}

void rust_helper_eth_hw_addr_set(struct net_device *dev, const u8 *addr)
{
	return eth_hw_addr_set(dev, addr);
}

void rust_helper_skb_tx_timestamp(struct sk_buff *skb)
{
	skb_tx_timestamp(skb);
}

void rust_helper_set_skb_ip_summed(struct sk_buff *skb, unsigned char ip_summed)
{
	skb->ip_summed = ip_summed;
}

void rust_helper_set_skb_protocol(struct sk_buff *skb, unsigned short protocol)
{
	skb->protocol = protocol;
}

void rust_helper_kfree_skb_reason(struct sk_buff *skb,
				  enum skb_drop_reason reason)
{
	sk_skb_reason_drop(NULL, skb, reason);
}

struct sk_buff *rust_helper_netdev_alloc_skb_ip_align(struct net_device *dev,
		unsigned int length)
{
	return netdev_alloc_skb_ip_align(dev, length);
}

void rust_helper_netif_start_queue(struct net_device *dev) {
	return netif_start_queue(dev);
}

void rust_helper_netif_stop_queue(struct net_device *dev) {
	return netif_stop_queue(dev);
}

bool rust_helper_netif_queue_stopped(struct net_device *dev) {
	return netif_queue_stopped(dev);
}

void rust_helper_netif_wake_queue(struct net_device *dev) {
	return netif_wake_queue(dev);
}
