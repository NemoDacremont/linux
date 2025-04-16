#include <asm-generic/socket.h>
#include <errno.h>
#include <netinet/in.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <sys/socket.h>
#include <netpacket/packet.h>
#include <netinet/ip.h>
#include <sys/ioctl.h>
#include <net/if.h>
#include <arpa/inet.h>
#include<net/ethernet.h>

// udp packet, broadcast ethernet, ip: dummy -> 192.168.252.2 (host) msg : tvalv2|recv\n
unsigned char bytes[] = {0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0x8, 0x0, 0x45, 0x0, 0x0, 0x28, 0xef, 0xc3, 0x40, 0x0, 0x40, 0x11, 0xd1, 0xab, 0xc0, 0xa8, 0xfc, 0x1, 0xc0, 0xa8, 0xfc, 0x2, 0xa0, 0x76, 0x27, 0xf, 0x0, 0x14, 0x2c, 0xe9, 0x74, 0x76, 0x61, 0x6c, 0x76, 0x32, 0x7c, 0x73, 0x65, 0x6e, 0x64, 0xa};


int main (int argc, char* argv[])
{
	int sockfd = socket(AF_PACKET, SOCK_RAW, htons(ETH_P_ALL));
	struct sockaddr_ll addr;
	addr.sll_family = AF_PACKET;
	addr.sll_protocol = htons(ETH_P_ALL);
	addr.sll_ifindex = if_nametoindex("eth0");

	for (int i = 0 ; i < sizeof(bytes) ; ++i)
		printf("%02x", bytes[i]);
	printf("\n");

	if (sendto(sockfd, bytes, sizeof(bytes), 0, (struct sockaddr*)&addr, sizeof(struct sockaddr_ll)) < 0)
		fprintf(stderr, "write failed: %d\n", errno);

	close(sockfd);
	return 0;
}
