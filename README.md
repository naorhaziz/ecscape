# ECSCape

**ECSCape** is a proof-of-concept exploit demonstrating a privilege escalation vulnerability in **Amazon ECS (EC2 launch type)**. It allows a low-privileged ECS task to hijack **IAM credentials** of other tasks running on the same container instance, breaking tenant isolation.

> **CVE Request Pending** | Research by [Naor Haziz](https://github.com/naorhaziz)

---

## About the Vulnerability

Amazon ECS (when using EC2 launch type) relies on the ECS Agent to communicate with AWS and relay IAM credentials to tasks using a WebSocket-based internal protocol called **ACS**.

**The flaw:** Any task on the same EC2 host can impersonate the ECS Agent and connect to the ACS endpoint using minimal permissions (`ecs:DiscoverPollEndpoint`). When it does, **ACS will deliver IAM credentials intended for other tasks**, resulting in a severe **cross-task credential leak**.

This project demonstrates that issue by replicating the agent's protocol, impersonating it, and dumping all received credentials.

---

## Usage

### Run as ECS Task in a Cluster

You can run `ecscape` directly as a task in your ECS cluster (EC2 launch type):

```json
{
  "family": "ecscape-task",
  "containerDefinitions": [
    {
      "name": "ecscape",
      "image": "naorhaziz/ecscape:latest",
      "essential": true,
      "memory": 128,
      "cpu": 100,
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/ecscape",
          "awslogs-region": "us-east-1",
          "awslogs-stream-prefix": "ecs"
        }
      },
      "networkMode": "bridge"
    }
  ]
}
```

> Ensure the task IAM role includes the `ecs:DiscoverPollEndpoint` permission.

Logs will be printed to **CloudWatch Logs** under `/ecs/ecscape`.

### Run via Docker (multi-arch)

```bash
# Pull and run the latest Docker image
sudo docker run --rm --net=host naorhaziz/ecscape:latest
```

> ⚠️ The container must run on an **EC2 instance managed by ECS** with other tasks running alongside it.

---

### Binary (Manual)

Prebuilt static binaries are available for:
- `x86_64-unknown-linux-musl`
- `aarch64-unknown-linux-musl`

You can download them from the [GitHub Actions Artifacts](https://github.com/naorhaziz/ecscape/actions).

Or build locally:
```bash
cargo build --release
./target/release/ecscape
```

---

## How It Works

1. Reads local EC2 instance metadata (IMDSv2)
2. Reads ECS Agent metadata (port `51678`)
3. Uses `ecs:DiscoverPollEndpoint` to get the ACS WebSocket URL
4. Signs the request using task credentials (SigV4)
5. Connects and impersonates the ECS Agent
6. Parses and prints `IAMRoleCredentialsMessage` packets

All received credentials are valid STS tokens assigned to **other tasks on the same instance**.

---

## Disclosure

This project is part of a responsible vulnerability disclosure submitted to **AWS Vulnerability Disclosure Program** via HackerOne.

### CVE Attribution Request

If this vulnerability qualifies for a CVE, credit is requested as:

> **Naor Haziz** — [https://github.com/naorhaziz](https://github.com/naorhaziz)

I am also open to coordinated disclosure and advisory review.

---

## License

This PoC is shared for educational and ethical research purposes under the [MIT License](LICENSE).

---

## Contact

- LinkedIn: [https://www.linkedin.com/in/naorhaziz](https://www.linkedin.com/in/naorhaziz)
- Twitter: [@naorhaziz](https://twitter.com/naorhaziz)
- GitHub: [https://github.com/naorhaziz](https://github.com/naorhaziz)

---

**Run responsibly. Only test on accounts and instances you own or have explicit permission to test.**
