// manual.js -- written by tools/manual.py from design/manual/ (do not edit): the apps' manual,
// its tours and the journey diagram (docs/RELEASE_PLAN.md P7).
// Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
export const MANUAL = {
 "images": {
  "journey": "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAxMDAwIDI1MCIgd2lkdGg9IjEwMDAiIGhlaWdodD0iMjUwIiBmb250LWZhbWlseT0iSW50ZXIsIEFyaWFsLCBzYW5zLXNlcmlmIiByb2xlPSJpbWciIGFyaWEtbGFiZWw9IlRoZSBqb3VybmV5IG9mIGEgbm9kZSI+CjxyZWN0IHdpZHRoPSIxMDAlIiBoZWlnaHQ9IjEwMCUiIGZpbGw9IiNmZmZmZmYiLz4KPGRlZnM+PG1hcmtlciBpZD0iYSIgdmlld0JveD0iMCAwIDEwIDEwIiByZWZYPSI5IiByZWZZPSI1IiBtYXJrZXJXaWR0aD0iNyIgbWFya2VySGVpZ2h0PSI3IiBvcmllbnQ9ImF1dG8iPjxwYXRoIGQ9Ik0wLDAgTDEwLDUgTDAsMTAgeiIgZmlsbD0iIzQ3NTU2OSIvPjwvbWFya2VyPjwvZGVmcz4KPHRleHQgeD0iNTAwLjAiIHk9IjI2IiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBmb250LXNpemU9IjE3IiBmb250LXdlaWdodD0iNjAwIiBmaWxsPSIjMGYxNzJhIj5Gcm9tIGFuIGF1dGhvciB0byBhIHJlbGVhc2U8L3RleHQ+CjxyZWN0IHg9IjIxIiB5PSI0NiIgd2lkdGg9IjExOCIgaGVpZ2h0PSI5MiIgcng9IjEwIiBmaWxsPSIjZTBmMmZlIiBzdHJva2U9IiMzMzQxNTUiIHN0cm9rZS13aWR0aD0iMS4yIi8+Cjx0ZXh0IHg9IjgwIiB5PSI3NiIgdGV4dC1hbmNob3I9Im1pZGRsZSIgZm9udC1zaXplPSIxNSIgZm9udC13ZWlnaHQ9IjYwMCIgZmlsbD0iIzBmMTcyYSI+MSBXcml0ZTwvdGV4dD4KPHRleHQgeD0iODAiIHk9IjEwMCIgdGV4dC1hbmNob3I9Im1pZGRsZSIgZm9udC1zaXplPSIxMiIgZmlsbD0iIzMzNDE1NSI+YXV0aG9yPC90ZXh0Pgo8dGV4dCB4PSI4MCIgeT0iMTIwIiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBmb250LXNpemU9IjEyIiBmaWxsPSIjNjQ3NDhiIj5ub2RlIGFwcDwvdGV4dD4KPHJlY3QgeD0iMTYxIiB5PSI0NiIgd2lkdGg9IjExOCIgaGVpZ2h0PSI5MiIgcng9IjEwIiBmaWxsPSIjZTBmMmZlIiBzdHJva2U9IiMzMzQxNTUiIHN0cm9rZS13aWR0aD0iMS4yIi8+Cjx0ZXh0IHg9IjIyMCIgeT0iNzYiIHRleHQtYW5jaG9yPSJtaWRkbGUiIGZvbnQtc2l6ZT0iMTUiIGZvbnQtd2VpZ2h0PSI2MDAiIGZpbGw9IiMwZjE3MmEiPjIgQ2hlY2s8L3RleHQ+Cjx0ZXh0IHg9IjIyMCIgeT0iMTAwIiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBmb250LXNpemU9IjEyIiBmaWxsPSIjMzM0MTU1Ij5zZWNvbmQgZW5naW5lZXI8L3RleHQ+Cjx0ZXh0IHg9IjIyMCIgeT0iMTIwIiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBmb250LXNpemU9IjEyIiBmaWxsPSIjNjQ3NDhiIj5ub2RlIGFwcDwvdGV4dD4KPGxpbmUgeDE9IjE0MSIgeTE9IjkyLjAiIHgyPSIxNTkiIHkyPSI5Mi4wIiBzdHJva2U9IiM0NzU1NjkiIHN0cm9rZS13aWR0aD0iMS42IiBtYXJrZXItZW5kPSJ1cmwoI2EpIi8+CjxyZWN0IHg9IjMwMSIgeT0iNDYiIHdpZHRoPSIxMTgiIGhlaWdodD0iOTIiIHJ4PSIxMCIgZmlsbD0iI2VkZTlmZSIgc3Ryb2tlPSIjMzM0MTU1IiBzdHJva2Utd2lkdGg9IjEuMiIvPgo8dGV4dCB4PSIzNjAiIHk9Ijc2IiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBmb250LXNpemU9IjE1IiBmb250LXdlaWdodD0iNjAwIiBmaWxsPSIjMGYxNzJhIj4zIFNpZ24gc3RhZ2U8L3RleHQ+Cjx0ZXh0IHg9IjM2MCIgeT0iMTAwIiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBmb250LXNpemU9IjEyIiBmaWxsPSIjMzM0MTU1Ij5zdGFnZSBvd25lcjwvdGV4dD4KPHRleHQgeD0iMzYwIiB5PSIxMjAiIHRleHQtYW5jaG9yPSJtaWRkbGUiIGZvbnQtc2l6ZT0iMTIiIGZpbGw9IiM2NDc0OGIiPmdyb3VwIGFwcDwvdGV4dD4KPGxpbmUgeDE9IjI4MSIgeTE9IjkyLjAiIHgyPSIyOTkiIHkyPSI5Mi4wIiBzdHJva2U9IiM0NzU1NjkiIHN0cm9rZS13aWR0aD0iMS42IiBtYXJrZXItZW5kPSJ1cmwoI2EpIi8+CjxyZWN0IHg9IjQ0MSIgeT0iNDYiIHdpZHRoPSIxMTgiIGhlaWdodD0iOTIiIHJ4PSIxMCIgZmlsbD0iI2VkZTlmZSIgc3Ryb2tlPSIjMzM0MTU1IiBzdHJva2Utd2lkdGg9IjEuMiIvPgo8dGV4dCB4PSI1MDAiIHk9Ijc2IiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBmb250LXNpemU9IjE1IiBmb250LXdlaWdodD0iNjAwIiBmaWxsPSIjMGYxNzJhIj40IFNlYWw8L3RleHQ+Cjx0ZXh0IHg9IjUwMCIgeT0iMTAwIiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBmb250LXNpemU9IjEyIiBmaWxsPSIjMzM0MTU1Ij5ncm91cCBsZWFkPC90ZXh0Pgo8dGV4dCB4PSI1MDAiIHk9IjEyMCIgdGV4dC1hbmNob3I9Im1pZGRsZSIgZm9udC1zaXplPSIxMiIgZmlsbD0iIzY0NzQ4YiI+Z3JvdXAgYXBwPC90ZXh0Pgo8bGluZSB4MT0iNDIxIiB5MT0iOTIuMCIgeDI9IjQzOSIgeTI9IjkyLjAiIHN0cm9rZT0iIzQ3NTU2OSIgc3Ryb2tlLXdpZHRoPSIxLjYiIG1hcmtlci1lbmQ9InVybCgjYSkiLz4KPHJlY3QgeD0iNTgxIiB5PSI0NiIgd2lkdGg9IjExOCIgaGVpZ2h0PSI5MiIgcng9IjEwIiBmaWxsPSIjZjFmNWY5IiBzdHJva2U9IiMzMzQxNTUiIHN0cm9rZS13aWR0aD0iMS4yIi8+Cjx0ZXh0IHg9IjY0MCIgeT0iNzYiIHRleHQtYW5jaG9yPSJtaWRkbGUiIGZvbnQtc2l6ZT0iMTUiIGZvbnQtd2VpZ2h0PSI2MDAiIGZpbGw9IiMwZjE3MmEiPjUgQnVpbGQ8L3RleHQ+Cjx0ZXh0IHg9IjY0MCIgeT0iMTAwIiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBmb250LXNpemU9IjEyIiBmaWxsPSIjMzM0MTU1Ij5kZXZlbG9wZXIgdGVhbTwvdGV4dD4KPHRleHQgeD0iNjQwIiB5PSIxMjAiIHRleHQtYW5jaG9yPSJtaWRkbGUiIGZvbnQtc2l6ZT0iMTIiIGZpbGw9IiM2NDc0OGIiPnJlcG9zaXRvcnk8L3RleHQ+CjxsaW5lIHgxPSI1NjEiIHkxPSI5Mi4wIiB4Mj0iNTc5IiB5Mj0iOTIuMCIgc3Ryb2tlPSIjNDc1NTY5IiBzdHJva2Utd2lkdGg9IjEuNiIgbWFya2VyLWVuZD0idXJsKCNhKSIvPgo8cmVjdCB4PSI3MjEiIHk9IjQ2IiB3aWR0aD0iMTE4IiBoZWlnaHQ9IjkyIiByeD0iMTAiIGZpbGw9IiNmMWY1ZjkiIHN0cm9rZT0iIzMzNDE1NSIgc3Ryb2tlLXdpZHRoPSIxLjIiLz4KPHRleHQgeD0iNzgwIiB5PSI3NiIgdGV4dC1hbmNob3I9Im1pZGRsZSIgZm9udC1zaXplPSIxNSIgZm9udC13ZWlnaHQ9IjYwMCIgZmlsbD0iIzBmMTcyYSI+NiBBY2NlcHQ8L3RleHQ+Cjx0ZXh0IHg9Ijc4MCIgeT0iMTAwIiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBmb250LXNpemU9IjEyIiBmaWxsPSIjMzM0MTU1Ij5ncm91cCBsZWFkPC90ZXh0Pgo8dGV4dCB4PSI3ODAiIHk9IjEyMCIgdGV4dC1hbmNob3I9Im1pZGRsZSIgZm9udC1zaXplPSIxMiIgZmlsbD0iIzY0NzQ4YiI+dGVzdCBhcHA8L3RleHQ+CjxsaW5lIHgxPSI3MDEiIHkxPSI5Mi4wIiB4Mj0iNzE5IiB5Mj0iOTIuMCIgc3Ryb2tlPSIjNDc1NTY5IiBzdHJva2Utd2lkdGg9IjEuNiIgbWFya2VyLWVuZD0idXJsKCNhKSIvPgo8cmVjdCB4PSI4NjEiIHk9IjQ2IiB3aWR0aD0iMTE4IiBoZWlnaHQ9IjkyIiByeD0iMTAiIGZpbGw9IiNmMWY1ZjkiIHN0cm9rZT0iIzMzNDE1NSIgc3Ryb2tlLXdpZHRoPSIxLjIiLz4KPHRleHQgeD0iOTIwIiB5PSI3NiIgdGV4dC1hbmNob3I9Im1pZGRsZSIgZm9udC1zaXplPSIxNSIgZm9udC13ZWlnaHQ9IjYwMCIgZmlsbD0iIzBmMTcyYSI+NyBSZWxlYXNlPC90ZXh0Pgo8dGV4dCB4PSI5MjAiIHk9IjEwMCIgdGV4dC1hbmNob3I9Im1pZGRsZSIgZm9udC1zaXplPSIxMiIgZmlsbD0iIzMzNDE1NSI+b3duZXI8L3RleHQ+Cjx0ZXh0IHg9IjkyMCIgeT0iMTIwIiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBmb250LXNpemU9IjEyIiBmaWxsPSIjNjQ3NDhiIj5tYWluPC90ZXh0Pgo8bGluZSB4MT0iODQxIiB5MT0iOTIuMCIgeDI9Ijg1OSIgeTI9IjkyLjAiIHN0cm9rZT0iIzQ3NTU2OSIgc3Ryb2tlLXdpZHRoPSIxLjYiIG1hcmtlci1lbmQ9InVybCgjYSkiLz4KPHBhdGggZD0iTTIyMCwxMzggVjE3MiBIODAgVjE0MiIgZmlsbD0ibm9uZSIgc3Ryb2tlPSIjOTRhM2I4IiBzdHJva2Utd2lkdGg9IjEuNCIgc3Ryb2tlLWRhc2hhcnJheT0iNSA0IiBtYXJrZXItZW5kPSJ1cmwoI2EpIi8+Cjx0ZXh0IHg9IjE1MCIgeT0iMTY2IiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBmb250LXNpemU9IjEyIiBmaWxsPSIjNDc1NTY5Ij5hbiBlZGl0IHRha2VzIHRoZSBzaWduYXR1cmUgb2ZmPC90ZXh0Pgo8cGF0aCBkPSJNNTAwLDEzOCBWMjA4IEg4MCBWMTQyIiBmaWxsPSJub25lIiBzdHJva2U9IiM5NGEzYjgiIHN0cm9rZS13aWR0aD0iMS40IiBzdHJva2UtZGFzaGFycmF5PSI1IDQiIG1hcmtlci1lbmQ9InVybCgjYSkiLz4KPHRleHQgeD0iMjkwIiB5PSIyMDIiIHRleHQtYW5jaG9yPSJtaWRkbGUiIGZvbnQtc2l6ZT0iMTIiIGZpbGw9IiM0NzU1NjkiPnJlLWlzc3VlIG9wZW5zIGEgc2VhbGVkIG5vZGUgYWdhaW48L3RleHQ+CjxwYXRoIGQ9Ik03ODAsMTM4IFYxNzIgSDUwMCBWMTQyIiBmaWxsPSJub25lIiBzdHJva2U9IiM5NGEzYjgiIHN0cm9rZS13aWR0aD0iMS40IiBzdHJva2UtZGFzaGFycmF5PSI1IDQiIG1hcmtlci1lbmQ9InVybCgjYSkiLz4KPHRleHQgeD0iNjQwIiB5PSIxNjYiIHRleHQtYW5jaG9yPSJtaWRkbGUiIGZvbnQtc2l6ZT0iMTIiIGZpbGw9IiM0NzU1NjkiPm5vdCBhY2NlcHRlZDogYmFjayB3aXRoIHJlYXNvbnM8L3RleHQ+Cjwvc3ZnPgo="
 },
 "pages": [
  {
   "app": "",
   "blocks": [
    [
     "p",
     [
      [
       "b",
       "In one line:"
      ],
      [
       "t",
       " an author writes a node in the node app, a second engineer checks it, the stage owner signs the stage, the group lead seals the group's release, and the developer team turns sealed releases into tested software that the lead accepts before it reaches everyone."
      ]
     ]
    ],
    [
     "img",
     "journey",
     "The journey"
    ],
    [
     "h2",
     [
      [
       "t",
       "Say it simply"
      ]
     ]
    ],
    [
     "p",
     [
      [
       "t",
       "Think of a chapter of a reference book. One person writes a section, a colleague reads it against the sources, the chapter editor signs the chapter, and the editor in chief prints the edition. Nothing reaches the printed book without passing every desk, and every desk leaves its name."
      ]
     ]
    ],
    [
     "p",
     [
      [
       "b",
       "Where the story lies:"
      ],
      [
       "t",
       " a book's section only has to read well. A node also computes: its code is generated from what the author wrote and tested against the answers the author gave from outside the code. So \"checked\" here means the numbers hold, not only the words."
      ]
     ]
    ],
    [
     "h2",
     [
      [
       "t",
       "Now the real thing"
      ]
     ]
    ],
    [
     "table",
     [
      [
       [
        "t",
        "Step"
       ]
      ],
      [
       [
        "t",
        "Who"
       ]
      ],
      [
       [
        "t",
        "Where"
       ]
      ],
      [
       [
        "t",
        "What leaves it"
       ]
      ]
     ],
     [
      [
       [
        [
         "t",
         "1 Write"
        ]
       ],
       [
        [
         "t",
         "the node's author"
        ]
       ],
       [
        [
         "t",
         "node app"
        ]
       ],
       [
        [
         "t",
         "a node marked ready, with no problem left on Review"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "2 Check"
        ]
       ],
       [
        [
         "t",
         "a second engineer (never the author)"
        ]
       ],
       [
        [
         "t",
         "node app, Review"
        ]
       ],
       [
        [
         "t",
         "a \"checked by\" signature over the node's content"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "3 Sign the stage"
        ]
       ],
       [
        [
         "t",
         "the stage's owner"
        ]
       ],
       [
        [
         "t",
         "group app, Assemble"
        ]
       ],
       [
        [
         "t",
         "a stage signature over all its nodes as they are"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "4 Seal"
        ]
       ],
       [
        [
         "t",
         "the group lead"
        ]
       ],
       [
        [
         "t",
         "group app, Release"
        ]
       ],
       [
        [
         "t",
         "a frozen release, "
        ],
        [
         "code",
         "releases/\u003cgroup>-\u003cversion>.tnrel"
        ],
        [
         "t",
         "; every node file sealed"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "5 Build"
        ]
       ],
       [
        [
         "t",
         "the developer team"
        ]
       ],
       [
        [
         "t",
         "the repository"
        ]
       ],
       [
        [
         "t",
         "the group's code generated from its pseudocode, tested against its test vectors, in a test app"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "6 Accept"
        ]
       ],
       [
        [
         "t",
         "the group lead"
        ]
       ],
       [
        [
         "t",
         "the test app"
        ]
       ],
       [
        [
         "t",
         "the group accepted, or sent back with what is wrong"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "7 Release"
        ]
       ],
       [
        [
         "t",
         "the owner"
        ]
       ],
       [
        [
         "code",
         "main"
        ]
       ],
       [
        [
         "t",
         "a release of the software everyone uses, naming each group's release"
        ]
       ]
      ]
     ]
    ],
    [
     "p",
     [
      [
       "t",
       "Every step can be undone by the one before it: an edit takes a signature off, a re-issue opens a sealed node again, and a group that is not accepted goes back with its reasons."
      ]
     ]
    ],
    [
     "p",
     [
      [
       "b",
       "Confirmed or UNCONFIRMED."
      ],
      [
       "t",
       " A node is sealed as confirmed only when steps 1 to 3 hold for it as it is now, the checks find nothing, and, if it computes, a test vector has its answer from outside the code. Every other node is sealed UNCONFIRMED with its reasons, and the software shows it as such. Nothing is hidden: a gap is visible, not missing."
      ]
     ]
    ],
    [
     "h2",
     [
      [
       "t",
       "Where to go next"
      ]
     ]
    ],
    [
     "ul",
     [
      [
       [
        "t",
        "If you write nodes: "
       ],
       [
        "b",
        "Guide for authors"
       ],
       [
        "t",
        "."
       ]
      ],
      [
       [
        "t",
        "If you lead a group or own a stage: "
       ],
       [
        "b",
        "Guide for group leads"
       ],
       [
        "t",
        "."
       ]
      ],
      [
       [
        "t",
        "If you build the software: "
       ],
       [
        "b",
        "Guide for developers"
       ],
       [
        "t",
        "."
       ]
      ],
      [
       [
        "t",
        "If you use the software's results: "
       ],
       [
        "b",
        "Guide for users"
       ],
       [
        "t",
        "."
       ]
      ]
     ]
    ]
   ],
   "file": "00_journey.md",
   "id": "journey",
   "role": "everyone",
   "title": "The journey of a node"
  },
  {
   "app": "node",
   "blocks": [
    [
     "p",
     [
      [
       "b",
       "In one line:"
      ],
      [
       "t",
       " you fill the node your lead issued to you, step by step, in TRI-NETRA Node; when Review shows nothing left to fix you mark it ready, and a second engineer signs it as checked."
      ]
     ]
    ],
    [
     "h2",
     [
      [
       "t",
       "Say it simply"
      ]
     ]
    ],
    [
     "p",
     [
      [
       "t",
       "The node app is a form that checks itself as you type. Each field asks one question, says why it is asked and shows an example. What is still missing is listed on Review, each item with a button that takes you to the field."
      ]
     ]
    ],
    [
     "p",
     [
      [
       "b",
       "Where the story lies:"
      ],
      [
       "t",
       " a form usually only wants its boxes filled. These checks also run your pseudocode on your test vectors and compare units, so a filled form can still have problems: Review is the judge, not the number of filled boxes."
      ]
     ]
    ],
    [
     "h2",
     [
      [
       "t",
       "Your first node"
      ]
     ]
    ],
    [
     "ol",
     [
      [
       [
        "b",
        "Open the app."
       ],
       [
        "t",
        " Open "
       ],
       [
        "code",
        "Apps/TRI-NETRA Node.html"
       ],
       [
        "t",
        " in Chrome or Edge, from the Drive folder on your computer (Drive for desktop). It runs offline; nothing leaves your computer."
       ]
      ],
      [
       [
        "b",
        "Say who you are."
       ],
       [
        "t",
        " Type your name the first time. It goes beside every change and signature."
       ]
      ],
      [
       [
        "b",
        "Open the design folder."
       ],
       [
        "t",
        " Choose "
       ],
       [
        "b",
        "Open design folder"
       ],
       [
        "t",
        " and pick the "
       ],
       [
        "code",
        "Design"
       ],
       [
        "t",
        " folder. The nodes issued to you are listed first."
       ]
      ],
      [
       [
        "b",
        "Open your node."
       ],
       [
        "t",
        " Click it. "
       ],
       [
        "b",
        "Home"
       ],
       [
        "t",
        " shows where it sits (its group, what it reads, who reads it), how far it is filled, and your lead's comments."
       ]
      ],
      [
       [
        "b",
        "Start from the spec."
       ],
       [
        "t",
        " When the spec already says something about this row, Home offers "
       ],
       [
        "b",
        "Start from the spec"
       ],
       [
        "t",
        ". Take it: those become your fields, and you can change any of them."
       ]
      ],
      [
       [
        "b",
        "Go through the steps."
       ],
       [
        "t",
        " Each tab is one step. Fill the fields; leaving a field saves it into the file's history (Undo takes it back)."
       ]
      ],
      [
       [
        "b",
        "Look at Review."
       ],
       [
        "t",
        " Every problem is listed with its rule (for example "
       ],
       [
        "code",
        "X01"
       ],
       [
        "t",
        ") and a "
       ],
       [
        "b",
        "Go there"
       ],
       [
        "t",
        " button. Fix them until the list is empty."
       ]
      ],
      [
       [
        "b",
        "Mark ready."
       ],
       [
        "t",
        " On Review, "
       ],
       [
        "b",
        "Mark ready"
       ],
       [
        "t",
        ". Then "
       ],
       [
        "b",
        "Save"
       ],
       [
        "t",
        "."
       ]
      ],
      [
       [
        "b",
        "Ask a colleague to check it."
       ],
       [
        "t",
        " They open the same node, read it on "
       ],
       [
        "b",
        "Preview"
       ],
       [
        "t",
        ", and choose "
       ],
       [
        "b",
        "Sign as checked"
       ],
       [
        "t",
        ". The app refuses your own name there."
       ]
      ]
     ]
    ],
    [
     "h2",
     [
      [
       "t",
       "Now the real thing"
      ]
     ]
    ],
    [
     "p",
     [
      [
       "b",
       "The kind decides the steps."
      ],
      [
       "t",
       " A declared node states a value with its source; a computed node is a relation from other nodes, with pseudocode and test vectors; a KPI states a requirement and which way it binds; an evidence node states what a test shows. Closures, interfaces and target rows are fixed by the tree: you write only their explanation and feedback. A row the spec has not named yet asks you to choose declared or computed first."
      ]
     ]
    ],
    [
     "p",
     [
      [
       "b",
       "Test vectors are the heart of a computed node."
      ],
      [
       "t",
       " Each one gives inputs, the expected answer, a tolerance and where the answer comes from: a page of a book, an independent derivation, another tool, a physical bound. An answer the code made proves nothing, and a computed node without an outside answer is never sealed as confirmed. "
      ],
      [
       "b",
       "Try it"
      ],
      [
       "t",
       " runs your pseudocode on every vector."
      ]
     ]
    ],
    [
     "p",
     [
      [
       "b",
       "Signatures cover what is written."
      ],
      [
       "t",
       " Marking ready and checking each record a fingerprint of the node's content. Any later edit makes them stale, and the app says so: mark it ready again and ask for the check again."
      ]
     ]
    ],
    [
     "p",
     [
      [
       "b",
       "Sealed means frozen."
      ],
      [
       "t",
       " When your lead seals the group, the node opens read-only and says which release sealed it. Your lead re-issues it when it is to change."
      ]
     ]
    ],
    [
     "p",
     [
      [
       "b",
       "Contracts."
      ],
      [
       "t",
       " When a node you read changes its contract, Home shows "
      ],
      [
       "b",
       "Acknowledge"
      ],
      [
       "t",
       ". When you need a contract changed, ask on Review ("
      ],
      [
       "b",
       "Ask for a contract change"
      ],
      [
       "t",
       "); the owning group's lead decides."
      ]
     ]
    ],
    [
     "h2",
     [
      [
       "t",
       "When something goes wrong"
      ]
     ]
    ],
    [
     "table",
     [
      [
       [
        "t",
        "What you see"
       ]
      ],
      [
       [
        "t",
        "What it means"
       ]
      ],
      [
       [
        "t",
        "What to do"
       ]
      ]
     ],
     [
      [
       [
        [
         "t",
         "Read-only: open for editing by someone"
        ]
       ],
       [
        [
         "t",
         "Someone else has the node open"
        ]
       ],
       [
        [
         "t",
         "Wait, or ask them to close it; the banner names them"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Read-only: sealed in a release"
        ]
       ],
       [
        [
         "t",
         "Your lead sealed the group"
        ]
       ],
       [
        [
         "t",
         "Ask your lead to re-issue it"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Unsaved work from before"
        ]
       ],
       [
        [
         "t",
         "The browser or computer stopped before a save"
        ]
       ],
       [
        [
         "b",
         "Restore them"
        ],
        [
         "t",
         ", then Save"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "A conflict copy beside the file"
        ]
       ],
       [
        [
         "t",
         "Drive saw two versions at once"
        ]
       ],
       [
        [
         "t",
         "Open both, keep the right one, tell your lead"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Signatures stale"
        ]
       ],
       [
        [
         "t",
         "You edited after ready or check"
        ]
       ],
       [
        [
         "t",
         "Mark ready again; ask for the check again"
        ]
       ]
      ]
     ]
    ]
   ],
   "file": "01_author.md",
   "id": "author",
   "role": "author",
   "title": "Guide for authors"
  },
  {
   "app": "group",
   "blocks": [
    [
     "p",
     [
      [
       "b",
       "In one line:"
      ],
      [
       "t",
       " you shape your group in TRI-NETRA Group (its nodes, stages, people and contracts), issue nodes to authors, follow their progress, and seal the group's release when it is ready; stage owners sign their stages before you seal."
      ]
     ]
    ],
    [
     "h2",
     [
      [
       "t",
       "Say it simply"
      ]
     ]
    ],
    [
     "p",
     [
      [
       "t",
       "You are the editor of one chapter. The map shows every section and what reads what. You hand sections to writers, see who is done, and when the chapter is ready you print an edition of it. An edition never changes; the next one gets a new number."
      ]
     ]
    ],
    [
     "p",
     [
      [
       "b",
       "Where the story lies:"
      ],
      [
       "t",
       " a printed edition cannot be corrected. A sealed release cannot either, but any node in it can be re-issued for the next release, and a node file that is lost can be made again from any release."
      ]
     ]
    ],
    [
     "h2",
     [
      [
       "t",
       "Your first week"
      ]
     ]
    ],
    [
     "ol",
     [
      [
       [
        "b",
        "Open the app."
       ],
       [
        "t",
        " Open "
       ],
       [
        "code",
        "Apps/TRI-NETRA Group.html"
       ],
       [
        "t",
        " in Chrome or Edge from the Drive folder on your computer, type your name, choose "
       ],
       [
        "b",
        "Open design folder"
       ],
       [
        "t",
        " and pick "
       ],
       [
        "code",
        "Design"
       ],
       [
        "t",
        "."
       ]
      ],
      [
       [
        "b",
        "Open your group."
       ],
       [
        "t",
        " Click it in the list. The banner says whether every structure rule holds."
       ]
      ],
      [
       [
        "b",
        "Add your people."
       ],
       [
        "t",
        " "
       ],
       [
        "b",
        "People → Add or change a member"
       ],
       [
        "t",
        ": yourself as "
       ],
       [
        "b",
        "lead"
       ],
       [
        "t",
        ", your stage owners, your authors. Names must be typed the same way everyone types them in the apps."
       ]
      ],
      [
       [
        "b",
        "Set the stage owners."
       ],
       [
        "t",
        " "
       ],
       [
        "b",
        "Stages"
       ],
       [
        "t",
        ", click a stage, choose its owner."
       ]
      ],
      [
       [
        "b",
        "Issue nodes."
       ],
       [
        "t",
        " On the "
       ],
       [
        "b",
        "Map"
       ],
       [
        "t",
        ", click a node, "
       ],
       [
        "b",
        "Issue…"
       ],
       [
        "t",
        ", choose the author. Their node app lists it first."
       ]
      ],
      [
       [
        "b",
        "Follow progress."
       ],
       [
        "t",
        " "
       ],
       [
        "b",
        "Progress"
       ],
       [
        "t",
        " shows each node's work (shell, draft, ready, checked), its signatures, its problems and its evidence debt."
       ]
      ],
      [
       [
        "b",
        "Comment."
       ],
       [
        "t",
        " Click a node (Map, Progress or Assemble) and "
       ],
       [
        "b",
        "Comment…"
       ],
       [
        "t",
        ": the author sees it on Home."
       ]
      ]
     ]
    ],
    [
     "h2",
     [
      [
       "t",
       "Now the real thing"
      ]
     ]
    ],
    [
     "p",
     [
      [
       "b",
       "Every change shows its impact first."
      ],
      [
       "t",
       " Each line is a note, a change, or a stop. "
      ],
      [
       "b",
       "Do it"
      ],
      [
       "t",
       " appears only when nothing stops it, and the action changes every file it touches or none."
      ]
     ]
    ],
    [
     "p",
     [
      [
       "b",
       "Another group's file changes only with its agreement."
      ],
      [
       "t",
       " Moving a node into another group, or archiving or merging a node another group reads, needs a change request accepted by that group's lead. The impact check offers to raise it."
      ]
     ]
    ],
    [
     "p",
     [
      [
       "b",
       "Releasing."
      ]
     ]
    ],
    [
     "ol",
     [
      [
       [
        "b",
        "Assemble"
       ],
       [
        "t",
        " lists the checks across your nodes and, for every node, why it is not yet confirmed."
       ]
      ],
      [
       [
        "t",
        "Each "
       ],
       [
        "b",
        "stage owner"
       ],
       [
        "t",
        " signs their stage ("
       ],
       [
        "b",
        "Assemble → Sign…"
       ],
       [
        "t",
        "). A change to any of its nodes takes the signature off."
       ]
      ],
      [
       [
        "t",
        "You "
       ],
       [
        "b",
        "seal"
       ],
       [
        "t",
        " ("
       ],
       [
        "b",
        "Release → Seal 1.0…"
       ],
       [
        "t",
        "). Only the lead can. The impact check says how many nodes go in confirmed and which do not."
       ]
      ]
     ]
    ],
    [
     "p",
     [
      [
       "t",
       "A node is sealed as confirmed only when a second engineer checked it as it is, the checks find nothing, its stage is signed and, if it computes, a test vector has its answer from outside the code. The rest go in UNCONFIRMED, each with why. Sealing writes "
      ],
      [
       "code",
       "releases/\u003cgroup>-\u003cversion>.tnrel"
      ],
      [
       "t",
       ", frozen, and seals every node file."
      ]
     ]
    ],
    [
     "p",
     [
      [
       "b",
       "After a release."
      ],
      [
       "t",
       " "
      ],
      [
       "b",
       "Re-issue"
      ],
      [
       "t",
       " opens a sealed node again (as it is, or as any release had it). "
      ],
      [
       "b",
       "Compare"
      ],
      [
       "t",
       " shows what changed between two releases, or since the last one. "
      ],
      [
       "b",
       "Import node forms"
      ],
      [
       "t",
       " takes today's node forms into your node files, keeping what authors typed."
      ]
     ]
    ],
    [
     "h2",
     [
      [
       "t",
       "For a stage owner"
      ]
     ]
    ],
    [
     "p",
     [
      [
       "t",
       "You sign your stage on "
      ],
      [
       "b",
       "Assemble"
      ],
      [
       "t",
       " when its nodes are as they should be. Your signature covers them exactly as they are; any change takes it off, and you sign again. Only you can sign your stage."
      ]
     ]
    ],
    [
     "h2",
     [
      [
       "t",
       "When something goes wrong"
      ]
     ]
    ],
    [
     "table",
     [
      [
       [
        "t",
        "What you see"
       ]
      ],
      [
       [
        "t",
        "What it means"
       ]
      ],
      [
       [
        "t",
        "What to do"
       ]
      ]
     ],
     [
      [
       [
        [
         "t",
         "An action stops: someone has a file open"
        ]
       ],
       [
        [
         "t",
         "An author is editing a node the action touches"
        ]
       ],
       [
        [
         "t",
         "Ask them to close it, then do it again"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "A structure action did not finish"
        ]
       ],
       [
        [
         "t",
         "The computer stopped in the middle"
        ]
       ],
       [
        [
         "b",
         "Finish it"
        ],
        [
         "t",
         " completes it from its record"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Seal stops: nothing changed"
        ]
       ],
       [
        [
         "t",
         "No node changed since the last release"
        ]
       ],
       [
        [
         "t",
         "Nothing to do"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Seal stops: a node file cannot be opened"
        ]
       ],
       [
        [
         "t",
         "A file is missing or damaged"
        ]
       ],
       [
        [
         "b",
         "Re-issue"
        ],
        [
         "t",
         " it from a release"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Fingerprints: problems"
        ]
       ],
       [
        [
         "t",
         "A release file was changed after sealing"
        ]
       ],
       [
        [
         "t",
         "Tell the developer team; do not edit release files"
        ]
       ]
      ]
     ]
    ]
   ],
   "file": "02_lead.md",
   "id": "lead",
   "role": "lead",
   "title": "Guide for group leads"
  },
  {
   "app": "",
   "blocks": [
    [
     "p",
     [
      [
       "b",
       "In one line:"
      ],
      [
       "t",
       " the developer team turns each group's sealed release into tested software: it checks the release, generates every computing row's code from its pseudocode, tests it against the release's own test vectors, and hands the group a test app to accept."
      ]
     ]
    ],
    [
     "h2",
     [
      [
       "t",
       "Say it simply"
      ]
     ]
    ],
    [
     "p",
     [
      [
       "t",
       "The groups write the recipes; you run the kitchen. You never change a recipe yourself: when one is wrong you send it back to its group. What you own is the kitchen, the tools and the checks that prove each dish matches its recipe."
      ]
     ]
    ],
    [
     "p",
     [
      [
       "b",
       "Where the story lies:"
      ],
      [
       "t",
       " a kitchen can taste and adjust. You cannot adjust a node's numbers to make a test pass; an expected value never comes from the code under test."
      ]
     ]
    ],
    [
     "h2",
     [
      [
       "t",
       "Now the real thing"
      ]
     ]
    ],
    [
     "p",
     [
      [
       "b",
       "What arrives."
      ],
      [
       "t",
       " A sealed release file, "
      ],
      [
       "code",
       "releases/\u003cgroup>-\u003cversion>.tnrel"
      ],
      [
       "t",
       ". "
      ],
      [
       "code",
       "python3 tools/release.py check"
      ],
      [
       "t",
       " checks its fingerprints, that its nodes are the group's, that each confirmed node was checked by someone other than its author and, when it computes, has an outside test vector, and that the lead sealed it."
      ]
     ]
    ],
    [
     "p",
     [
      [
       "b",
       "What you do with it."
      ],
      [
       "t",
       " The release is merged into "
      ],
      [
       "code",
       "design.tndb"
      ],
      [
       "t",
       ", every computing row's code is generated from its pseudocode (Rust for the engine, MATLAB for the twin) and tested against the release's test vectors, the C and Rust flight software are held to the pseudocode interpreter, and the group gets a test app to accept."
      ]
     ]
    ],
    [
     "p",
     [
      [
       "b",
       "Where things are."
      ],
      [
       "t",
       " The design files and their schema: "
      ],
      [
       "code",
       "design/schema.toml"
      ],
      [
       "t",
       ", "
      ],
      [
       "code",
       "tools/tndb.py"
      ],
      [
       "t",
       ". The structure rules: "
      ],
      [
       "code",
       "tools/group.py"
      ],
      [
       "t",
       ". Releases: "
      ],
      [
       "code",
       "tools/release.py"
      ],
      [
       "t",
       ". The pseudocode: "
      ],
      [
       "code",
       "docs/PSEUDOCODE_V2.md"
      ],
      [
       "t",
       ", "
      ],
      [
       "code",
       "tools/pcode.py"
      ],
      [
       "t",
       ". Every command: "
      ],
      [
       "code",
       "docs/COMMANDS.md"
      ],
      [
       "t",
       ". Every check in one command: "
      ],
      [
       "code",
       "python3 tools/check_all.py"
      ],
      [
       "t",
       "."
      ]
     ]
    ],
    [
     "p",
     [
      [
       "b",
       "The rules nothing bends."
      ],
      [
       "t",
       " Node content comes only from its group's release. An expected value never comes from the code under test. A computing node without an outside answer is never confirmed. Every check that can fail fails by name."
      ]
     ]
    ],
    [
     "p",
     [
      [
       "t",
       "The developer manual in full is in "
      ],
      [
       "code",
       "spec/manual/developer/"
      ],
      [
       "t",
       "."
      ]
     ]
    ]
   ],
   "file": "03_developer.md",
   "id": "developer",
   "role": "developer",
   "title": "Guide for developers"
  },
  {
   "app": "",
   "blocks": [
    [
     "p",
     [
      [
       "b",
       "In one line:"
      ],
      [
       "t",
       " you use the software by writing one case (your satellite, its orbit and mission, what its ADCS must achieve) and reading the result it gives back; every node behind a result shows whether it was confirmed by its group or is still UNCONFIRMED."
      ]
     ]
    ],
    [
     "h2",
     [
      [
       "t",
       "Say it simply"
      ]
     ]
    ],
    [
     "p",
     [
      [
       "t",
       "You hand in a question (your case) and get back a report (the result). The report names the edition of every chapter it used, and marks anything that was not confirmed."
      ]
     ]
    ],
    [
     "p",
     [
      [
       "b",
       "Where the story lies:"
      ],
      [
       "t",
       " a report from a book is only as good as the book. Here each number also says how far to trust it: a confirmed node was checked by a second engineer against answers from outside the code; an UNCONFIRMED one was not, and the result says so."
      ]
     ]
    ],
    [
     "h2",
     [
      [
       "t",
       "Now the real thing"
      ]
     ]
    ],
    [
     "ul",
     [
      [
       [
        "b",
        "Your case"
       ],
       [
        "t",
        " is the only thing you edit. The software never guesses a number you left blank: it names the key and stops."
       ]
      ],
      [
       [
        "b",
        "The result"
       ],
       [
        "t",
        " is one file holding every number and plot of the run. Opening it runs nothing."
       ]
      ],
      [
       [
        "b",
        "Asking for something different"
       ],
       [
        "t",
        " is a node form sent to the developer team, or a word with the group lead who owns the node."
       ]
      ],
      [
       [
        "b",
        "Which release."
       ],
       [
        "t",
        " The software shows which release of each group it contains; the release notes list every node still UNCONFIRMED."
       ]
      ]
     ]
    ],
    [
     "p",
     [
      [
       "t",
       "The user manual in full is in "
      ],
      [
       "code",
       "spec/manual/user/"
      ],
      [
       "t",
       ", starting with "
      ],
      [
       "i",
       "Start here"
      ],
      [
       "t",
       "."
      ]
     ]
    ]
   ],
   "file": "04_user.md",
   "id": "user",
   "role": "user",
   "title": "Guide for users"
  },
  {
   "app": "",
   "blocks": [
    [
     "p",
     [
      [
       "b",
       "In one line:"
      ],
      [
       "t",
       " the words the apps use, each in one sentence."
      ]
     ]
    ],
    [
     "table",
     [
      [
       [
        "t",
        "Word"
       ]
      ],
      [
       [
        "t",
        "What it means"
       ]
      ]
     ],
     [
      [
       [
        [
         "t",
         "Node"
        ]
       ],
       [
        [
         "t",
         "One row of the design tree: a value, a relation, a requirement, a piece of evidence, a closure or an interface, in its own file"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Node file"
        ]
       ],
       [
        [
         "code",
         "nodes/\u003cid>.node.tndb"
        ],
        [
         "t",
         ": everything its author writes and signs"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Group"
        ]
       ],
       [
        [
         "t",
         "One of the 20 parts of the design, led by one lead; its file is "
        ],
        [
         "code",
         "structure/\u003cgroup>.group.tndb"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Stage"
        ]
       ],
       [
        [
         "t",
         "A part of a group with its own owner, who signs it"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Lead"
        ]
       ],
       [
        [
         "t",
         "The person who shapes the group, issues its nodes and seals its releases"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Stage owner"
        ]
       ],
       [
        [
         "t",
         "The person who signs a stage"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Author"
        ]
       ],
       [
        [
         "t",
         "The person a node is issued to; only they change it, until it is sealed"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Checker"
        ]
       ],
       [
        [
         "t",
         "A second engineer, never the author, who signs a node as checked"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Issue"
        ]
       ],
       [
        [
         "t",
         "Give a node to an author"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Ready"
        ]
       ],
       [
        [
         "t",
         "The author's signature: nothing left to fix"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Checked"
        ]
       ],
       [
        [
         "t",
         "The checker's signature over the node as it is"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Stale"
        ]
       ],
       [
        [
         "t",
         "A signature that no longer covers the node, because it changed after"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Test vector"
        ]
       ],
       [
        [
         "t",
         "Inputs, the expected answer, a tolerance, and where the answer comes from"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Outside answer"
        ]
       ],
       [
        [
         "t",
         "An expected value from a book, an independent derivation, another tool or a physical bound, never from the code under test"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Evidence debt"
        ]
       ],
       [
        [
         "t",
         "What a node claims without evidence yet"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Contract"
        ]
       ],
       [
        [
         "t",
         "An output another group reads, with its unit and version"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Change request"
        ]
       ],
       [
        [
         "t",
         "A request to another group to change something it owns"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Impact check"
        ]
       ],
       [
        [
         "t",
         "What an action would change, shown before it is done; a stop blocks it"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Assemble"
        ]
       ],
       [
        [
         "t",
         "The whole group read and checked across its nodes"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Seal"
        ]
       ],
       [
        [
         "t",
         "The lead freezes the group as a numbered release"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Release"
        ]
       ],
       [
        [
         "code",
         "releases/\u003cgroup>-\u003cversion>.tnrel"
        ],
        [
         "t",
         ": the group as sealed, never edited"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Confirmed"
        ]
       ],
       [
        [
         "t",
         "Sealed with every check passing and signed by a checker and its stage owner"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "UNCONFIRMED"
        ]
       ],
       [
        [
         "t",
         "Sealed with at least one reason not to trust it yet, named in the release"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Re-issue"
        ]
       ],
       [
        [
         "t",
         "Open a sealed node again, as it is or as a release had it"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Node form"
        ]
       ],
       [
        [
         "t",
         "Today's request file ("
        ],
        [
         "code",
         "adcs-node-form/1"
        ],
        [
         "t",
         "), which the group app can import"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Fingerprint"
        ]
       ],
       [
        [
         "t",
         "A SHA-256 hash: if one byte changes, the fingerprint changes"
        ]
       ]
      ]
     ]
    ]
   ],
   "file": "05_glossary.md",
   "id": "glossary",
   "role": "everyone",
   "title": "Glossary"
  },
  {
   "app": "files",
   "blocks": [
    [
     "p",
     [
      [
       "b",
       "In one line:"
      ],
      [
       "t",
       " TRI-NETRA Files opens any one design file, lets you change it with undo, saves it safely, and shows its history; it is the tool for looking inside a file, not for writing nodes or leading a group."
      ]
     ]
    ],
    [
     "h2",
     [
      [
       "t",
       "Say it simply"
      ]
     ]
    ],
    [
     "p",
     [
      [
       "t",
       "It is a careful text editor for design files. It refuses to overwrite a file someone else has open, keeps your unsaved work if the computer stops, and checks every save by reading it back."
      ]
     ]
    ],
    [
     "p",
     [
      [
       "b",
       "Where the story lies:"
      ],
      [
       "t",
       " a text editor lets you type anything. This one only writes what the design files' format allows, and the node app and group app hold their files to more rules than it does. Use those apps for their work."
      ]
     ]
    ],
    [
     "h2",
     [
      [
       "t",
       "Now the real thing"
      ]
     ]
    ],
    [
     "ol",
     [
      [
       [
        "b",
        "Open a folder or a file."
       ],
       [
        "t",
        " "
       ],
       [
        "b",
        "Open folder"
       ],
       [
        "t",
        " lists every design file in it, with who has one open and any conflict copies. "
       ],
       [
        "b",
        "Open a file"
       ],
       [
        "t",
        " opens one."
       ]
      ],
      [
       [
        "b",
        "Change, undo, save."
       ],
       [
        "t",
        " Every change is one step you can undo. "
       ],
       [
        "b",
        "Save"
       ],
       [
        "t",
        " writes the file, reads it back and compares it; a file changed on disk since you opened it is never overwritten: your work goes to a named copy."
       ]
      ],
      [
       [
        "b",
        "History."
       ],
       [
        "t",
        " Every save is a revision: who, when, what. Drive keeps every saved version too."
       ]
      ]
     ]
    ],
    [
     "table",
     [
      [
       [
        "t",
        "What you see"
       ]
      ],
      [
       [
        "t",
        "What it means"
       ]
      ],
      [
       [
        "t",
        "What to do"
       ]
      ]
     ],
     [
      [
       [
        [
         "t",
         "Read-only: open for editing by someone"
        ]
       ],
       [
        [
         "t",
         "Someone else has it open"
        ]
       ],
       [
        [
         "t",
         "Wait, or ask them to close it"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "Unsaved work from before"
        ]
       ],
       [
        [
         "t",
         "The computer stopped before a save"
        ]
       ],
       [
        [
         "b",
         "Restore them"
        ],
        [
         "t",
         ", then Save"
        ]
       ]
      ],
      [
       [
        [
         "t",
         "A newer format"
        ]
       ],
       [
        [
         "t",
         "The file was written by a newer TRI-NETRA"
        ]
       ],
       [
        [
         "t",
         "Use the newer apps"
        ]
       ]
      ]
     ]
    ]
   ],
   "file": "06_files.md",
   "id": "files",
   "role": "everyone",
   "title": "Guide to TRI-NETRA Files"
  }
 ],
 "tours": {
  "files-start": {
   "app": "files",
   "steps": [
    {
     "target": "open-folder",
     "text": "Pick the folder on your computer that holds the design files (with Drive for desktop, your Drive folder is on your computer). Every file in it is listed.",
     "title": "Open the folder"
    },
    {
     "target": "open-file",
     "text": "Open a single design file instead. You change it, undo, save, and see its history.",
     "title": "Or open one file"
    },
    {
     "target": "help",
     "text": "Guides for every role, the glossary, the journey of a node, and this tour again.",
     "title": "Help is always here"
    }
   ]
  },
  "group-list": {
   "app": "group",
   "steps": [
    {
     "target": "groups",
     "text": "Every group of the design, with its nodes and the change requests waiting for it. Click yours.",
     "title": "The groups"
    },
    {
     "target": "check-all",
     "text": "Holds every group and node file to the structure rules, and names anything broken.",
     "title": "Check the whole design"
    }
   ]
  },
  "group-open": {
   "app": "group",
   "steps": [
    {
     "target": "tabs",
     "text": "Map: the nodes, their stages and what reads what; click a node to change it or issue it. Every change shows its impact first.",
     "title": "Your group"
    },
    {
     "target": "[data-tab=\"People\"]",
     "text": "Add yourself as lead, your stage owners and authors. Names must be typed as they type them in the apps.",
     "title": "People"
    },
    {
     "target": "[data-tab=\"Progress\"]",
     "text": "Every node's work, signatures, problems and evidence debt, and what changed since the last release.",
     "title": "Progress"
    },
    {
     "target": "[data-tab=\"Assemble\"]",
     "text": "The checks across your nodes, the stage signatures, and why each node is not yet confirmed.",
     "title": "Assemble"
    },
    {
     "target": "[data-tab=\"Release\"]",
     "text": "Seal the group's release (only the lead), compare releases, re-issue a node, import node forms.",
     "title": "Release"
    }
   ]
  },
  "group-start": {
   "app": "group",
   "steps": [
    {
     "target": "open-folder",
     "text": "Pick the Design folder: the one holding structure/ and nodes/. Your name is asked once and written beside every change.",
     "title": "Open the design folder"
    },
    {
     "target": "help",
     "text": "The guide for group leads and stage owners, the glossary, the journey of a node to a release, and this tour again.",
     "title": "Help is always here"
    }
   ]
  },
  "node-list": {
   "app": "node",
   "steps": [
    {
     "target": "mine",
     "text": "The nodes your group lead issued to you. Click one to open it.",
     "title": "Issued to you"
    },
    {
     "target": "search",
     "text": "By its id, its label, its group, or an author's name. You can open any node; only yours are yours to change.",
     "title": "Find any node"
    }
   ]
  },
  "node-open": {
   "app": "node",
   "steps": [
    {
     "target": "steps",
     "text": "Home first: where the node sits, how far it is, your lead's comments. Then one tab per step, each field with its question, why it is asked and an example.",
     "title": "One tab per step"
    },
    {
     "target": "[data-tab^=\"Review\"]",
     "text": "Every problem the checks find, each with a Go there button. When the list is empty, Mark ready. A colleague then signs it as checked; never you.",
     "title": "Review"
    },
    {
     "target": "[data-tab=\"Preview\"]",
     "text": "The node as its readers will see it, answer first. Print it from here.",
     "title": "Preview"
    },
    {
     "target": "save",
     "text": "Leaving a field keeps the change (Undo takes it back); Save writes the file, reads it back and checks it. Unsaved work survives a crash and is offered back.",
     "title": "Save"
    }
   ]
  },
  "node-start": {
   "app": "node",
   "steps": [
    {
     "target": "open-folder",
     "text": "Pick the Design folder: the one holding structure/ and nodes/. The nodes issued to you are listed first. Your name is asked once and written beside every change.",
     "title": "Open the design folder"
    },
    {
     "target": "help",
     "text": "The guide for authors, the glossary, the journey of a node from you to a release, and this tour again.",
     "title": "Help is always here"
    }
   ]
  }
 }
};
