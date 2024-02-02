from io import StringIO
import os.path
from pyinfra.operations import files, systemd, apt

# To get just the directory containing the script, use os.path.dirname
script_dir = os.path.dirname(os.path.abspath(__file__))

# Define your DNS service details
dns_service_name = 'dnsR'
dns_service_executable = os.path.join(script_dir, 'target/release/dns_r')  # Path to your DNS executable
dns_server_ip = '127.0.0.1:1053'  # IP of your DNS server
dns_domains = 'superfruitmix.dev.'  # Domain your DNS server should handle

#Ensure systemd-resolved is installed
apt.packages(
    name='Install systemd-resolved',
    packages=['systemd-resolved'],
    update=True,
    _sudo=True
)

#Ensure systemd-resolved is enabled and running
systemd.service(
    name='Enable and start systemd-resolved',
    service='systemd-resolved',
    running=True,
    enabled=True,
    _sudo=True
)

#Ensure resolved.conf.d directory exists
files.directory(
    name='Ensure resolved.conf.d directory exists',
    path='/etc/systemd/resolved.conf.d',
    present=True,
    _sudo=True
)

systemd_resolved_conf = f'/etc/systemd/resolved.conf.d/{dns_service_name}.conf'

# Create systemd service file for your DNS service
systemd_service_content = StringIO(f'''
[Unit]
Description={dns_service_name}
After=network.target

[Service]
ExecStart={dns_service_executable}
WorkingDirectory={script_dir}
Restart=on-failure

[Install]
WantedBy=multi-user.target
''')

files.template(
    name='Setup systemd service file for DNS',
    src=systemd_service_content,
    dest=f'/etc/systemd/system/{dns_service_name}.service',
    _sudo=True
)
# Enable and start the custom DNS service
systemd.service(
    name=f'Enable and start {dns_service_name}',
    service=dns_service_name,
    running=True,
    enabled=True,
    _sudo=True
)

# Configure systemd-resolved
files.line(
    name='Configure systemd-resolved for custom DNS',
    path=systemd_resolved_conf,
    line=f'[Resolve]',
    _sudo=True
)

# Configure systemd-resolved
files.line(
    name='Configure systemd-resolved for custom DNS',
    path=systemd_resolved_conf,
    line=f'DNS={dns_server_ip}',
    _sudo=True
)

# Configure systemd-resolved
files.line(
    name='Configure systemd-resolved for custom DNS',
    path=systemd_resolved_conf,
    line=f'Domains=~{dns_domains}',
    _sudo=True
)

# Restart systemd-resolved to apply changes
systemd.service(
    name='Restart systemd-resolved',
    service='systemd-resolved',
    restarted=True,
    _sudo=True,
)