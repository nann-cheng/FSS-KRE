
# old way, is now obsoleted.

# ./clear.sh
# # Create two namespaces: Let's name them ns1 and ns2.
# sudo ip netns add ns1
# sudo ip netns add ns2

# #Step 2: Create Virtual Ethernet (veth) Pairs

# sudo ip link add veth1 type veth peer name veth2

# sudo ip link set veth1 netns ns1
# sudo ip link set veth2 netns ns2


# #Step 3: Configure IP Addresses

# sudo ip netns exec ns1 ip addr add 192.168.1.1/24 dev veth1
# sudo ip netns exec ns2 ip addr add 192.168.1.2/24 dev veth2

# sudo ip netns exec ns1 ip link set veth1 up
# sudo ip netns exec ns2 ip link set veth2 up

# #Step 4: Apply Traffic Control (tc)

# # LAN
# # sudo ip netns exec ns1 tc qdisc add dev veth1 root netem delay 0.1ms rate 1gbit
# sudo ip netns exec ns1 tc qdisc add dev veth1 root netem delay 80ms rate 100mbit
# # sudo ip netns exec ns1 tc qdisc add dev veth1 root netem delay 80ms rate 100mbit



#Step 5: Apply Traffic Control (tc)

# sudo ip netns exec ns1 ping 192.168.1.2

sudo tc qdisc add dev lo root handle 1: netem delay 40ms