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
