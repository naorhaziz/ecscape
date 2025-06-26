# ECSCape

**ECSCape** is a proof-of-concept exploit demonstrating a privilege escalation vulnerability in **Amazon ECS (EC2 launch type)**. It allows a low-privileged ECS task to hijack **IAM credentials** of other tasks running on the same container instance, breaking tenant isolation.

> ⚠️ The container must run on an **EC2 instance managed by ECS** with other tasks running alongside it.
> Research by [Naor Haziz](https://github.com/naorhaziz)

## License

This PoC is shared for educational and ethical research purposes under the [MIT License](LICENSE).

---

## Contact

- LinkedIn: [https://www.linkedin.com/in/naorhaziz](https://www.linkedin.com/in/naorhaziz)
- Twitter: [@naorhaziz](https://twitter.com/naorhaziz)
- GitHub: [https://github.com/naorhaziz](https://github.com/naorhaziz)

---

**Run responsibly. Only test on accounts and instances you own or have explicit permission to test.**
