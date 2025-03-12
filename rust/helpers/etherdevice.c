#include <linux/etherdevice.h>
#include <linux/skbuff.h>

struct net_device *rust_helper_alloc_etherdev(int sizeof_priv)
{
	return alloc_etherdev(sizeof_priv);
}

void *rust_helper_netdev_priv(const struct net_device *dev)
{
	return netdev_priv(dev);
}

void rust_helper_eth_hw_addr_set(struct net_device *dev, const u8 *addr)
{
	return eth_hw_addr_set(dev, addr);
}

void rust_helper_eth_hw_addr_random(struct net_device *dev)
{
	eth_hw_addr_random(dev);
}

void rust_helper_skb_tx_timestamp(struct sk_buff *skb)
{
	skb_tx_timestamp(skb);
}
