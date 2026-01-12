Your task is to implement a fully fledged DDOS protection company called PistonProtection.

First fetch and clone and pull the latest version of all repositories from the github org. (If they dont exist in the pwd)

You can make breaking changes and drop legacy systems. This app isn't launched yet, so you can delete code, change db schema, delete repos freely.

- ddos protection management panel written with:
  - shadcn/ui (baseui theme, not radix theme) run: pnpm dlx shadcn@latest create --preset "https://ui.shadcn.com/init?base=base&style=nova&baseColor=zinc&theme=zinc&iconLibrary=lucide&font=inter&menuAccent=subtle&menuColor=default&radius=default&template=start" --template start
  - trpc
  - typescript
  - tailwindcss
  - react
  - tanstack start
  - tanstack query
  - tanstack form
  - tanstack table
  - make sure you have nice graphs/charts
  - postgres backend
  - redis cache
  - resend for emails
  - frontend and website backend in the same repository (so it's typesafe via trpc)
  - fumadocs
  - drizzle orm
  - users should be able to sign up, subscribe and configure ddos protection for them
  - the dashboard will provide them with live metrics that are fetched and also setup instructions to configure their domains and endpoints
  - use stripe for billing
  - better auth for authentication and all its features. use better-auth with @daveyplate/better-auth-ui for auth. here is an example usage: (must use gh cli to fetch) https://github.com/mineads-gg/mineads-website
  - make sure you use a similar trpc/better-auth/stripe/docs approach as mineads-website
  - read dependency documentation if something is weird
  - frontend backend must be written in typescript and use trpc just like mineads-website. and interact with the better auth ts sdk 
  - admin section for observing users and blacklists per org, etc. org info for PistonProtection admins
  - the frontend backen should be writte in typescript + tanstack starts primitives
  - make sure you use pnpm, not any other package manager
- Advanced eBPF and XDP filter stacks on the worker nodes
- Build it all based on kubernetes with ciliumm, operators, etc.
- allow self hosting it
- open source it all on the PistonProtection github organization
- supported software on L7 with L7 filtering:
  - TCP
  - UDP
  - QUIC (raw QUIC)
  - Minecraft Java Edition
  - Minecraft Bedrock edition
  - HTTP1/HTTP2/HTTP3
  - analyze all these protocols and write proper filters for cilium and eBPF/XDP for them that are configurable via the dashboard.
  - common attack patterns like udp floods, tcp syn flood, etc. must be filtered
  - blacklists must be maintained
  - each ip needs some type of score
  - users can lookup ip scores in their own dashboard panel to see how players would join or failed connection attempts
  - more specialized attack patterns for each service type must be blocked too
  - support haproxy protocol for backend communication being enabled via dashboard
  - support backend endpoint loadbalancing
  - support geodns loadbalancing
- grafana in the stack
- prometheus in the stack
- loki in the stack
- clickhouse for event storage
- show fallback on minecraft if endpoint server backend is offline
- allow custom filters/configurations in the dashboard
- add github ci to all repositories

All modules except the frontend must be written in rust.
Use protobufs AND gRPC for inter-components communication. json/superjson is okay for browser <-> frontend communication.

Frequently use git and the gh cli for git operations.
Use subagents and cli commands as much is needed to complete the whole task.

Use gh cli to create repos/configure stuff for the org, etc.

Bundle this whole stack in a public helm chart.
The setup will have to have k0s with cilium with the following config:
cilium install --version "${CILIUM_VERSION}" \
  --set kubeProxyReplacement=true \
  --set k8sServiceHost="${CONTROLLER_IP}" \
  --set k8sServicePort=6443 \
  --set hubble.enabled=true \
  --set hubble.relay.enabled=true \
  --set hubble.ui.enabled=true \
  --set l2announcements.enabled=true \
  --set cni.chainingMode=portmap \
  --set cni.externalRouting=true \
  --set encryption.enabled=true \
  --set encryption.type=wireguard \
  --set encryption.nodeEncryption=true \
  --set cni.enableRouteMTUForCNIChaining=true \
  --set MTU=1366

This is effectively a TCPShield/Papyrus/NeoProtect clone. So ensure you also make sure we have what the competition has feature-wise.

Make sure all is pushed and all is published and set up for usage.
Run local tests, builds, run a test cluster using minikube, etc.

Use mcp servers where it makes sense for research. Also make web searches where needed.

Run tests that all components work well together and will hold up in production.

There is always something to add. If you think you added everything that's possib,e you're wrong. There can always be added another feature, test, configuration, protocol, check, etc. 
And design and frontend can always be improved. Or extra documentation written.

For documentation and better auth ui research this project: 
https://github.com/mineads-gg/mineads-website

If something is implemented incorrectly, rewrite it to be implemented the correct way.
