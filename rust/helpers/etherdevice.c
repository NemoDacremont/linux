#include <linux/etherdevice.h>

struct net_device *rust_helper_alloc_etherdev(int sizeof_priv)
{
	return alloc_etherdev(sizeof_priv);
}