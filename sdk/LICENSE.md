# tessoa plugin SDK licence

Copyright 2026 Chengdu Tianji Engineering Management Service Co., Ltd.
(成都市天基工程管理服务有限公司). All rights reserved, except as granted below.

The SDK is the contents of this folder: the manual, the interface definition,
the API reference, the guides, the TypeScript declarations and the Rust SDK.
The tools are the tessoa-plugin command-line tool that the setup scripts
install, including the plugin runner inside it.

## What you may do

Free of charge, without asking us and without keeping any notice, you may:

1. use the SDK and the tools to write, build, test, package and install
   plugins for tessoa;
2. distribute the plugins you build to anyone, free or for a fee, under terms
   of your choice. This includes the parts of the SDK that a build puts into a
   plugin, such as the compiled Rust SDK in a Rust plugin;
3. keep this folder in your plugin's source repository, where the plugin
   template puts it, and share or publish that repository with the folder in
   it, privately or publicly;
4. make tools that help people build plugins for tessoa, such as editor
   extensions, checkers or templates of your own, and include the SDK in them.

## What you may not do

You may not:

1. distribute the SDK on its own, apart from a plugin's source repository or a
   tool described in item 4 above, or distribute a changed copy of the SDK as
   if it were the SDK we publish;
2. distribute the tools. People get them from us through the setup scripts,
   which check every download against the fingerprints in the plugin template;
3. use the SDK or the tools to make software that lets people use tessoa
   plugins outside tessoa, such as another application that loads them. Tools
   under item 4 above that run plugins only to test them are not affected.

## No warranty

The SDK and the tools are provided "as is", without warranty of any kind,
express or implied. To the extent permitted by applicable law, the copyright
holder is not liable for any claim, damages or other liability arising from
the SDK, the tools or their use.

## Other terms

People who install your plugin in tessoa do so under the tessoa Plugin Terms;
this licence covers only what you may do with the SDK and the tools. The
open-source components inside the tools come under their own licences, listed
in the THIRD-PARTY-NOTICES file that is installed with them. If this
licence and its Chinese version (LICENSE.zh-CN.md) differ, the Chinese version
prevails.
