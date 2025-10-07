# Index Media Server

**The fastest, simplest way to stream your personal media collection. No subscriptions, no tracking, no limits.**

[![License: MPL-2.0](https://img.shields.io/badge/License-MPL--2.0-red.svg)](https://opensource.org/licenses/MPL-2.0) [![Discord](https://img.shields.io/badge/Discord-Join%20Community-blue.svg)](https://discord.gg/WamXjEhcaa)

## What is Index Media Server?

Index Media Server is the server side to Index Stream, a **privacy-first, open-source media streaming platform** designed for people who want complete control over their personal media without the complexity and overhead of traditional solutions.

### Why Index Stream?

- 🚀 **Ease of Use** - Simple configurations with a modern UI
- 🔒 **Privacy First** - No tracking, no cloud dependencies for local connections, no signups necessary 
- 💰 **Truly Free** - No subscriptions, no premium tiers, no hidden costs

**The Result:** Minutes to setup instead of hours and giving you complete peace of mind about your privacy.

## Repository Structure

This repository contains the **media server** for Index Stream. 

- **Media Server (this repo)**: Tauri desktop application that serves your media files
- **Frontend**: [indexstream.org](https://github.com/index-stream/indexstream.org) - Web-based streaming interface and landing page

## Developer Quick Start

### Prerequisites
- Cargo 1.87+
- Node.js 23+
- npm

### Building and Running the Server

```bash
# Clone the repository
git clone https://github.com/index-stream/index-media-server.git
cd index-media-server

# Install dependencies
npm install

# Run in development mode
npm run dev

# (Optional) build the Tauri application
npm run build
```

The server will start and appear in your system tray. Click the tray icon to open the web interface.

## Project Status

🚧 **Currently in Alpha** - Expect breaking changes and rapid iteration.

We're actively developing core features and refining the user experience.

**Server Roadmap:**
- ✅ User setup flow
- ✅ Profile and Index Management
- ✅ Support local connections over HTTPS
- 📋 Automatic media indexing
- 📋 Endpoints for querying media against index
- 📋 Endpoints for serving media
- 📋 Media transcoding
- 📋 Remote access configuration
- 📋 Plugin support

## Community

💬 **Join our community on Discord** to discuss, ask questions, and contribute.

[![Discord](https://img.shields.io/badge/Discord-Join%20Community-blue.svg)](https://discord.gg/WamXjEhcaa)

We're building Index Stream with the community in mind. Whether you're a developer looking to contribute, a user with feedback, or someone interested in privacy-first media streaming, we'd love to have you join the conversation.

## Contributing

We welcome contributions! Whether you're fixing bugs, adding features, or improving documentation, your help makes Index Stream better for everyone.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the Mozilla Public License 2.0 - see the [LICENSE](LICENSE) file for details.

*Built with ❤️ for the privacy-conscious media streaming community.*
