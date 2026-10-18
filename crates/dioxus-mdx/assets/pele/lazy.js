/*! Pele 1.6.0
MIT License

Copyright (c) 2026 kepano

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

---

The reset icon of the zoom controls is "rotate-ccw" from Lucide v1.14.0
(https://lucide.dev), under the ISC License:

Copyright (c) 2026 Lucide Icons and Contributors

Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted, provided that the above
copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
*/
import{a as l,b as o,c as m,d as t,e as k,f as D,g as s,h as u}from"./chunks/ESCH476E.js";import"./chunks/EJ5KOXYD.js";import"./chunks/GUJTEFZK.js";import{a as w,h}from"./chunks/ND66TM4A.js";var g=new Map([["flowchart",()=>import("./chunks/NV4XNBIS.js").then(n=>n.flowchart)],["pie",()=>import("./chunks/XRRJ35LF.js").then(n=>n.pie)],["er",()=>import("./chunks/BNMLP6AG.js").then(n=>n.er)],["mindmap",()=>import("./chunks/KS3BB3KE.js").then(n=>n.mindmap)],["kanban",()=>import("./chunks/UHTQJXKL.js").then(n=>n.kanban)],["timeline",()=>import("./chunks/FP7KDXVC.js").then(n=>n.timeline)],["journey",()=>import("./chunks/MXGNCBL2.js").then(n=>n.journey)],["class",()=>import("./chunks/QYHQDLKZ.js").then(n=>n.classDiagram)],["quadrantChart",()=>import("./chunks/ZF3OVT4B.js").then(n=>n.quadrantChart)],["xychart",()=>import("./chunks/4TTDE5I4.js").then(n=>n.xychart)],["state",()=>import("./chunks/AAXQQPKT.js").then(n=>n.state)],["gantt",()=>import("./chunks/YFKBLVVN.js").then(n=>n.gantt)],["architecture",()=>import("./chunks/PU4ISKXV.js").then(n=>n.architecture)],["requirement",()=>import("./chunks/Q3IYVLG5.js").then(n=>n.requirement)],["sankey",()=>import("./chunks/DR6PKOQU.js").then(n=>n.sankey)],["packet",()=>import("./chunks/NU5M23IW.js").then(n=>n.packet)],["radar",()=>import("./chunks/ERTIBQVS.js").then(n=>n.radar)],["treemap",()=>import("./chunks/GY35JXOZ.js").then(n=>n.treemap)],["info",()=>import("./chunks/4UF3HTLV.js").then(n=>n.info)],["sequence",()=>import("./chunks/MWY2J62P.js").then(n=>n.sequence)],["gitGraph",()=>import("./chunks/OGLU4SIR.js").then(n=>n.gitGraph)],["c4",()=>import("./chunks/YHRE5QGZ.js").then(n=>n.c4)],["block",()=>import("./chunks/46IJ22JX.js").then(n=>n.block)],["railroad",()=>import("./chunks/2DXBBFLN.js").then(n=>n.railroad)],["agentflow",()=>import("./chunks/F653G4FO.js").then(n=>n.agentflow)],["treeView",()=>import("./chunks/VKNV3BUX.js").then(n=>n.treeView)],["cynefin",()=>import("./chunks/AKE5O7KZ.js").then(n=>n.cynefin)],["wardley",()=>import("./chunks/E4RJ7UJB.js").then(n=>n.wardley)],["eventmodeling",()=>import("./chunks/FT5I3RHX.js").then(n=>n.eventmodeling)],["usecase",()=>import("./chunks/OUNS5N76.js").then(n=>n.usecase)],["venn",()=>import("./chunks/KXJRRZHI.js").then(n=>n.venn)],["ishikawa",()=>import("./chunks/UXCHDUKG.js").then(n=>n.ishikawa)],["swimlane",()=>import("./chunks/H5EEJPP2.js").then(n=>n.swimlane)]]),i=new Map;function p(n){if(n===null)return Promise.resolve(!1);if(m(n))return Promise.resolve(!0);let a=i.get(n);if(a===void 0){let e=g.get(n);if(e===void 0)return Promise.resolve(!1);a=e().then(r=>(o(r),!0),r=>{throw i.delete(n),r}),i.set(n,a)}return a}async function d(n,a={}){let e=null;try{e=t(n)}catch{}return await p(e),s(n,a)}async function y(n,a,e={}){let r=null;try{r=t(a)}catch{}return await p(r),u(n,a,e)}export{w as PeleError,t as detectType,l as enableZoom,h as forgetTextWidths,p as load,u as mount,y as mountAsync,D as parse,o as register,m as registered,s as render,d as renderAsync,k as supports};
