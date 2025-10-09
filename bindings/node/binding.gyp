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
      "cflags_cc": [
        "-std=c++17"
      ],
      "defines": [
        "NAPI_DISABLE_CPP_EXCEPTIONS"
      ],
      "conditions": [
        [
          "OS=='win'",
          {
            "libraries": [
              "<(ZKPROV_STATIC)",
              "advapi32.lib",
              "bcrypt.lib",
              "user32.lib",
              "ws2_32.lib"
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
}
