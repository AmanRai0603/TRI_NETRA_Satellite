# Results
<!-- kind: how-to; depth: read -->

**In one line:** every finished run is one HTML file that holds everything and runs nothing; open it in any browser, read its first sentence and its "Where this result breaks", and keep it beside the case it ran on.

## Read a result

1. **Open the file** in any browser. It works offline, on any computer.
2. **Read the first sentence.** It says how many requirements pass, for which case and product, in how many runs, on which engine, and how far to trust it. A demonstration result says so in bold: it is never evidence.
3. **Read "How to read this result"** if the page is new to you: it maps the four parts, from the verdict down to the data.
4. **Look up the numbers** under "Requirements and metrics": each requirement, the achieved statistic at your level, the margin and the verdict (an icon and a word, never colour alone). "What a margin is" explains the arithmetic.
5. **Read "Where this result breaks"** before you use the result for anything. It lists what the model leaves out, the rows nobody has confirmed, and the levels your case did not state.
6. Switch the page to **Learn** to get the exercises, or **Expert** for the tables alone.

## Keep it with your other results

1. In the software, choose **Results → Open a result** and pick the file. It is filed in your store beside its case, added to your list, and shown. Nothing runs.
2. In MATLAB, `asils.result.import(file)` does the same into the zip's store; `rec = asils.result.open(file)` reads it back for plotting.
3. From a terminal, `adcs result import <file>`.

| You run in | The store is |
|---|---|
| the web app or the workbench | `~/.adcs/store/` on your computer |
| the MATLAB SILS tool | the zip's `store/` folder |
| the portal | your project |

```
store/
  cases/<case id>/<sha12>.csv                        your case, exactly as run
  results/<case id>/<sha12>/<scenario>_<date>.result.html
  candidates/<id>.toml                               products the designer found (design team)
  index.csv                                          a list of every result; rebuilt from the files
```

`<sha12>` names the exact version of your case, so a result is never shown beside a case it did not run on. The files are the record; `index.csv` is only a list, rebuilt from them. A new store already holds the release's reference cases.

## Compare two results

1. On a result page, press **Open a result to compare…** and pick another result file.
2. Its metrics appear beside these; its first channel is overlaid, dashed, on the same axes.
3. The other result may use a different case, product, engine or release. Comparing a MATLAB result with a platform result is how you see that the two tools agree.

## Share a result

1. Send the file. The person you send it to needs only a browser.
2. Results made for a client leave out restricted content, and say what was left out under "Where this came from".

## Run a result's case again

1. Under "Where this came from", open "The case, exactly as it was run" and press **Download the case CSV**.
2. Open that CSV in the software and run. A run of a campaign not kept in the file is re-created exactly from its seed with `adcs sim rerun <file> <k>`, the only command that runs the engine for a saved result.

## If it goes wrong

| You see | It means | Do |
|---|---|---|
| "Channels could not be read" | the file was cut short, or the browser is very old | ask the sender for the file again; use a current browser |
| "That file is not a result document" | you picked another kind of HTML file to compare | pick a `.result.html` |
| "This is a demonstration: it is not evidence" | the engine is `demo` | use it to learn the page, never to decide anything |
| the result looks wrong | a node, a product or the case may be wrong | send feedback with the result file attached ([Asking for changes](04_asking_for_changes.md)); the developer team opens exactly what you saw without re-running it |
