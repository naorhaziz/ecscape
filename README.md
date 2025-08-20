# ECSCape

**ECSCape** is a proof‑of‑concept exploit demonstrating a privilege‑escalation vulnerability in **Amazon ECS (EC2 launch type)**. It allows a low‑privileged ECS task to hijack **IAM credentials** of other tasks running on the same container instance, breaking task isolation. It can obtain the **task execution role** credentials (the agent‑only identity used by ECS to pull images, push logs, decrypt, etc.), which are **not intended to be accessible by containers**.

> ⚠️ Requirements:
> - Tasks must run on an **EC2 instance managed by ECS** (EC2 launch type), with other tasks running on the same host.
> - Not applicable to Fargate.

**Research:** [Naor Haziz](https://github.com/naorhaziz)

---

## What this PoC demonstrates

- **Cross‑task IAM credential theft (task role):** Steal credentials of other ECS tasks on the same EC2 host and use their permissions for lateral movement and privilege escalation.
- **Task execution role theft:** Obtain credentials intended for the **ECS agents** (e.g., to read from ECR, write CloudWatch Logs, KMS/Secrets Manager). This identity is **not meant for application containers** and can expand impact if mis‑scoped. Additionally, if any tasks on the same EC2 instance have secrets defined (via environment variables ValueFrom), this PoC can steal those secrets using the hijacked task execution role credentials, allowing access to **all container secrets** from Secrets Manager or Parameter Store configured for any task running on the compromised instance.

---

## License

This PoC is shared for educational and ethical research purposes under the [MIT License](LICENSE).

---

## Contact

- LinkedIn: [https://www.linkedin.com/in/naorhaziz](https://www.linkedin.com/in/naorhaziz)
- Twitter: [https://x.com/naor_haziz](https://x.com/naor_haziz)
- GitHub: [https://github.com/naorhaziz](https://github.com/naorhaziz)

---

**Run responsibly. Only test on accounts and instances you own or have explicit permission to test.**
