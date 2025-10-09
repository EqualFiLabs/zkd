{
  "targets": [
    {
      "target_name": "zkprov",
      "sources": [
        "src/addon.cc"
      ],
      "variables": {
        "ZKPROV_STATIC": "<!(node -p \"(() => { const p = process.env.ZKPROV_STATIC; if (!p) { throw new Error('ZKPROV_STATIC is not set'); } return p; })()\")"
      },
      "include_dirs": [
        "<!(node -p \"require('node-addon-api').include.replace(/\\\"/g, '')\")",
        "<!@(node -p \"require('node-addon-api').include_dir\")",
        "<(module_root_dir)/../../include"
      ],
      "defines": [
        "NAPI_DISABLE_CPP_EXCEPTIONS"
      ],
      "conditions": [
        [
          "OS=='win'",
          {
            "defines": [
              "_HAS_EXCEPTIONS=0"
            ],
            "libraries": [
              "<(ZKPROV_STATIC)",
              "advapi32.lib",
              "bcrypt.lib",
              "ntdll.lib",
              "userenv.lib",
              "user32.lib",
              "ws2_32.lib"
            ],
            "msvs_settings": {
              "VCCLCompilerTool": {
                "AdditionalOptions": [
                  "/std:c++17"
                ]
              }
            }
          },
          {
            "cflags_cc": [
              "-std=c++17"
            ],
            "conditions": [
              [
                "OS=='mac'",
                {
                  "libraries": [
                    "-Wl,-force_load",
                    "<(ZKPROV_STATIC)",
                    "-ldl",
                    "-lpthread",
                    "-lm"
                  ]
                },
                {
                  "libraries": [
                    "<(ZKPROV_STATIC)",
                    "-ldl",
                    "-lpthread",
                    "-lm"
                  ]
                }
              ]
            ]
          }
        ]
      ]
    }
  ]
}
