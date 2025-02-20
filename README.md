# Fluxa

Fluxa helps you ensure your services remain online by periodically
checking their availability and sending alerts if they go down.
Additionally, once a service recovers, you will receive a recovery notification.
With its low memory usage and efficient operation,
Fluxa is an ideal solution for monitoring primary services such
as Uptime Kuma and others.

By configuring the services, retries, and notifications through
the configuration file, you can easily manage the monitoring of multiple services.
Additionally, the built-in web server functionality allows you to perform
cross-monitoring and ensure Fluxa itself remains up and running.

![fluxa >](https://raw.githubusercontent.com/rbas/fluxa/main/assets/fluxa.webp)

## 🚀 Fluxa Features

* **Lightweight and Efficient:** Fluxa has very low RAM usage (in units of MB),
making it ideal for low-resource environments.
* **Reliable Monitoring:** Built to monitor primary monitoring
services like Uptime Kuma, but flexible enough to monitor any service by URL.
* **Minimal Footprint:** Fluxa is a small, reliable service designed to
run efficiently without consuming excessive resources.

## 🛠 Usage

Once **Fluxa** is installed and the configuration file is ready, you can run it
by executing the binary with the `--config` parameter, followed by the path
to your configuration file.

For example:

```shell
./fluxa --config /path/to/your/config.toml
```

This will start **Fluxa**, and it will begin monitoring the services defined in your
configuration file. Additionally, Fluxa's internal web server will be running
at the configured listen address (e.g., `127.0.0.1:8080`)
to allow cross-monitoring of *Fluxa* itself.

## 📦 Installation

You can install Fluxa in two ways: by downloading a pre-compiled binary
or by building it from source. Below are the instructions for both methods.

### 1. Download Pre-compiled Binaries

If you don't want to build the project from source, you can download
the latest release for your platform.

1. Visit the [latest releases page](https://github.com/rbas/fluxa/releases/latest) on GitHub.
2. Choose the appropriate binary for your platform:

    * For **macOS (arm64)**: `fluxa-aarch64-apple-darwin.tar.xz`
    * For **Linux (x86_64)**: `fluxa-x86_64-unknown-linux-gnu.tar.xz`

3. Download and Extract

    * Download the `.tar.xz` file for your platform.
    * Extract the contents of the archive

4. Run the Project: After extracting, navigate to the directory containing the binary and run the executable

```shell
./fluxa --config /path/to/config.toml
```

### 2. Install from Source (Using cargo build)

1. **Prerequisites**: Make sure you have [Rust installed](https://www.rust-lang.org/tools/install) on your machine.
2. **Clone the Repository**: Clone the repository to your local machine
3. **Build the Project**: Build the project using Cargo

```shell
cargo build --release
```

> *This will compile the project and generate an executable in the `target/release/` directory.*

4. **Run the Project**: After building, you can run the project:

```shell
./target/release/fluxa --config /path/to/config.toml
```

## ⚙️ Configuration

The configuration file is structured as follows:

1. **Pushover API Keys** (for notifications)
2. **Fluxa Settings** (to configure the service itself)
3. **Services** (list of monitored services and their specific settings)

### Pushover API Keys

Fluxa can send notifications through Pushover when a monitored service is down
or recovered. You need to provide the Pushover API key and user/group key
for sending notifications.

```toml
# Pushover API key
pushover_api_key = "api key"

# Pushover user or group key
pushover_user_key = "key"
```

* `pushover_api_key`: This is the API key provided by Pushover to authenticate the service.
* `pushover_user_key`: This key identifies the user or group that should receive the notifications.

### Fluxa Settings

Fluxa runs as a service that listens for incoming requests.
Below is the configuration for how Fluxa will behave when running.

```toml
[fluxa]
# Listen on
listen = "127.0.0.1:8080"
```

* `listen`: The address and port on which Fluxa will listen. In this example,
Fluxa listens on `127.0.0.1:8080`, meaning it will only accept local connections.
Adjust the address and port as needed.

#### Fluxa Health Check Endpoint

Fluxa provides an endpoint for cross-monitoring, which can be used to monitor the status of the Fluxa service itself. This endpoint responds with an HTTP status of `200`` and a body of`ok`. You can use this endpoint to monitor Fluxa using another monitoring system.

    Health Check URL: http://<fluxa_host>:<fluxa_port>/
    Response:
        HTTP Status: 200 OK
        Body: ok

You can use this endpoint to confirm that Fluxa is running and responsive.

### Service Configuration

The services section defines a list of URLs that Fluxa will monitor.
Each service can be configured with its monitoring interval,
retry mechanism, and maximum retry attempts.

#### Service Configuration Example

``` toml
[[services]]
# Monitored url
url = "http://localhost:3000"

# How often the url will be monitored (in seconds)
interval_seconds = 300

# Maximum number of retries before the service is considered down
max_retries = 3

# Retry interval (in seconds) before the next attempt
retry_interval = 3
```

#### Fields Description

* `url`: The URL of the service that Fluxa will monitor. Replace "<http://localhost:3000>" with the actual URL you want to monitor.
* `interval_seconds`: The frequency (in seconds) at which the URL will be checked. In the example, it is set to 300 seconds (or 5 minutes).
* `max_retries`: The number of retry attempts to make if the URL check fails. If the service fails max_retries times consecutively, it will be marked as down. In this example, it is set to 3 retries.
* `retry_interval`: The time (in seconds) Fluxa waits before retrying the check. For example, if this is set to 3, Fluxa will retry the check every 3 seconds.

#### Service Status Notifications

Fluxa sends notifications when a monitored service is down and when it recovers.

* **Service Down**: When Fluxa detects that a service has failed (i.e., after the service reaches the maximum retry attempts and still cannot be reached), it will send a single notification about the service being down.

* **Service Recovered**: Once the service is back online and successfully responds to the monitoring checks, Fluxa will send a notification indicating that the service has recovered.

These notifications are sent via the Pushover API, ensuring that the designated user or group receives an alert both when the service goes down and when it comes back online.

#### Notes

* **Multiple service blocks**: Each service can be defined separately using the `[[services]]` format. Fluxa can monitor an arbitrary number of services by adding multiple blocks in the configuration file.
* **Custom intervals**: You can configure different intervals and retry behaviors for each service according to its needs.
* **Single Notification**: For each service, only one notification is sent for each state change (down or recovered). Duplicate notifications are not sent for the same status.

## 🌈 Motivation

My motivation for creating Fluxa stemmed from the need for a lightweight, meta-monitoring tool designed to check whether other monitoring tools are up and running. With a focus on minimal memory footprint—just a few MB—Fluxa is built to be simple yet effective, capable of running even on resource-constrained devices like routers. The goal was to create a solution that could cross-monitor both main and advanced monitoring tools, ensuring seamless oversight without adding unnecessary complexity or overhead. Additionally, I integrated support for Pushover notifications to ensure timely alerts when services go down, making Fluxa not only a reliable monitoring tool but also a practical way to maintain uptime across critical systems.

## ⭐ Contribute

You are welcomed to fork the project and create a branch for each new feature
you wish to add. Ensure that you write necessary tests and run the current
tests before making your pull request.

## License: MIT

© 2025 Martin Voldrich
This work is licensed under [MIT license](https://github.com/rbas/fluxa/blob/main/LeCENSE).
