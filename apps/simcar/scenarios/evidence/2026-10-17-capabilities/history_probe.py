import time
import rclpy
from rclpy.qos import QoSProfile, ReliabilityPolicy, DurabilityPolicy
from std_msgs.msg import String

rclpy.init()
node = rclpy.create_node('capabilities_history_probe')
end = time.monotonic() + 5
while not node.get_publishers_info_by_topic('/rovi/capabilities') and time.monotonic() < end:
    rclpy.spin_once(node, timeout_sec=0.05)
info = node.get_publishers_info_by_topic('/rovi/capabilities')
assert len(info) == 1, info
assert info[0].qos_profile.durability == DurabilityPolicy.TRANSIENT_LOCAL, info
assert info[0].qos_profile.reliability == ReliabilityPolicy.RELIABLE, info
assert info[0].qos_profile.depth == 1, info
received = []
start = time.monotonic()
sub = node.create_subscription(String, '/rovi/capabilities', lambda m: received.append((time.monotonic() - start, m.data)), QoSProfile(depth=1, reliability=ReliabilityPolicy.RELIABLE, durability=DurabilityPolicy.TRANSIENT_LOCAL))
while time.monotonic() - start < 3:
    rclpy.spin_once(node, timeout_sec=0.05)
assert not received, ('retained history was delivered', received)
print('No retained sample within 3 s after late subscription.', flush=True)
while not received and time.monotonic() - start < 15:
    rclpy.spin_once(node, timeout_sec=0.05)
assert received, 'Live control did not arrive: cannot conclude that only history is missing'
print(f'Same transient-local publisher delivered its later live sample after {received[0][0]:.3f} s.', flush=True)
print(received[0][1], flush=True)
node.destroy_node()
rclpy.shutdown()
