import { createFromRoot } from 'codama';
import { rootNodeFromAnchor, type AnchorIdl } from
'@codama/nodes-from-anchor';
import { renderVisitor as renderJavaScriptVisitor } from
"@codama/renderers-js";
import anchorIdl from '../programs/Turbin3_prereq.json'
import path from 'path';
const codama = createFromRoot(rootNodeFromAnchor(anchorIdl as AnchorIdl));
const jsClient = path.join(import.meta.dirname, "..", "clients", "js");
codama.accept(renderJavaScriptVisitor(path.join(jsClient, "src", "generated")));