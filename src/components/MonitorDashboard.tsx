import { createSignal, onMount, onCleanup, Show, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

// ─── Types ───────────────────────────

interface ISSPos { latitude: number; longitude: number; altitude: number; velocity: number }
interface NewsItem { title: string; source: string; timestamp: string }
interface Activity { lat: number; lon: number; label: string; event_type: string; intensity: number }
interface SysStats { os: string; hostname: string; uptime_secs: number; cpu_count: number; memory_total_mb: number; memory_used_mb: number; local_ip: string; public_ip: string | null }

interface Props { onClose: () => void }

// ═══════════════════════════════════════
// DETAILED COASTLINE DATA  [lon, lat]
// ═══════════════════════════════════════

const COASTS: [number, number][][] = [

  // ── NORTH AMERICA ─────────────────

  // Alaska Aleutians & Peninsula
  [[-170,53],[-168,53],[-166,54],[-164,54],[-162,54],[-161,55],[-160,56],[-158,57],[-156,57],[-155,58],[-153,58],[-152,59],[-150,60],[-149,61],[-148,61],[-147,61],[-146,61]],

  // Alaska South Coast to Panhandle
  [[-146,61],[-145,60],[-144,60],[-143,60],[-142,60],[-141,60],[-140,60],[-139,59],[-138,58],[-137,58],[-136,58],[-135,57],[-134,56],[-133,56],[-132,55],[-131,54],[-130,55]],

  // Alaska North Coast
  [[-141,70],[-144,70],[-148,71],[-152,71],[-155,71],[-158,71],[-160,70],[-163,69],[-165,68],[-167,67],[-168,66],[-166,65],[-164,64],[-163,63],[-162,62],[-163,60],[-162,58],[-161,57],[-160,56]],

  // BC & Pacific NW
  [[-130,55],[-129,53],[-128,52],[-127,51],[-126,50],[-126,49],[-125,49],[-124,48],[-124,47],[-124,46]],

  // US West Coast
  [[-124,46],[-124,44],[-124,43],[-124,42],[-123,41],[-122,40],[-122,39],[-121,38],[-121,37],[-120,36],[-119,35],[-118,34],[-118,33],[-117,33],[-117,32]],

  // Baja & Mexico Pacific
  [[-117,32],[-116,31],[-115,30],[-114,29],[-113,28],[-112,27],[-111,26],[-110,25],[-109,24],[-108,23],[-107,22],[-106,21],[-105,20],[-105,19],[-104,18],[-103,17],[-102,17],[-101,17],[-100,17],[-98,16],[-97,16]],

  // Central America Pacific
  [[-97,16],[-95,15],[-94,15],[-93,14],[-92,14],[-91,13],[-90,13],[-89,13],[-88,13],[-87,12],[-86,11],[-85,10],[-84,10],[-83,9],[-82,8],[-81,8],[-80,7],[-79,7],[-78,6],[-77,7]],

  // South America Pacific Coast
  [[-77,7],[-78,4],[-79,2],[-80,1],[-80,0],[-81,-1],[-81,-3],[-81,-5],[-80,-6],[-79,-7],[-78,-8],[-77,-10],[-76,-12],[-76,-14],[-75,-15],[-75,-17],[-74,-18],[-72,-18],[-71,-19],[-70,-21],[-70,-23],[-70,-25],[-71,-28],[-71,-30],[-72,-32],[-73,-34],[-73,-36],[-74,-38],[-74,-40],[-74,-42],[-74,-44],[-73,-46],[-74,-48],[-73,-50],[-72,-52],[-71,-53],[-70,-54],[-69,-55],[-68,-56]],

  // Tierra del Fuego
  [[-68,-56],[-67,-55],[-66,-55],[-65,-55],[-64,-55]],

  // South America Atlantic (south to north)
  [[-64,-55],[-64,-52],[-63,-50],[-63,-48],[-62,-45],[-61,-42],[-60,-40],[-58,-38],[-57,-36],[-56,-35],[-55,-34],[-53,-33],[-52,-32],[-51,-30],[-50,-29],[-49,-28],[-48,-27],[-47,-25],[-46,-24],[-45,-23],[-44,-23],[-43,-23],[-42,-22],[-41,-21],[-40,-20],[-39,-18],[-39,-16],[-38,-14],[-38,-12],[-37,-10],[-36,-8],[-35,-6],[-35,-4],[-35,-2],[-36,0],[-38,0],[-40,1],[-44,2],[-48,2],[-50,3],[-52,4],[-54,5],[-56,6],[-58,7],[-60,8],[-61,9],[-62,10],[-64,11],[-67,11],[-68,12],[-70,12],[-72,11],[-73,11],[-75,10],[-77,9],[-78,9],[-79,9],[-80,8],[-80,7]],

  // Caribbean coast Colombia/Venezuela
  [[-77,9],[-76,10],[-75,11],[-74,11],[-73,12],[-71,12],[-70,12],[-68,11],[-67,11],[-65,10],[-64,10],[-63,10],[-62,11],[-61,10],[-60,11]],

  // Gulf of Mexico — Texas to Florida
  [[-97,16],[-97,18],[-97,20],[-97,22],[-97,24],[-97,26],[-96,28],[-95,29],[-94,30],[-93,30],[-92,29],[-91,29],[-90,29],[-89,29],[-89,30],[-88,30],[-87,30],[-86,30],[-85,30],[-84,30],[-83,29],[-82,28],[-82,27],[-82,26],[-81,25],[-80,25]],

  // Florida peninsula & US East Coast
  [[-80,25],[-80,26],[-80,27],[-81,28],[-81,30],[-81,31],[-80,32],[-79,33],[-78,34],[-77,35],[-76,36],[-76,37],[-75,38],[-75,39],[-74,40],[-74,41],[-73,41],[-72,41],[-71,42],[-70,42],[-70,43],[-69,44],[-68,44],[-67,45],[-67,44],[-66,44]],

  // Maritime Canada
  [[-66,44],[-65,44],[-64,45],[-63,46],[-62,46],[-61,46],[-60,47],[-59,47],[-58,48],[-57,48],[-56,48],[-55,47],[-53,47],[-52,47]],

  // Newfoundland
  [[-52,47],[-53,48],[-54,49],[-55,50],[-56,51],[-57,52],[-58,52],[-59,51],[-58,49],[-56,48],[-54,47],[-52,47]],

  // Labrador coast
  [[-56,51],[-58,53],[-59,55],[-60,56],[-61,57],[-62,58],[-64,60],[-66,60],[-68,61],[-70,62]],

  // East coast Hudson Bay to Arctic
  [[-70,62],[-72,62],[-74,63],[-76,63],[-78,62],[-80,60],[-82,58],[-84,57],[-86,56],[-88,56]],

  // Hudson Bay
  [[-88,56],[-86,55],[-84,54],[-82,53],[-80,54],[-78,55],[-76,56],[-76,58],[-78,60],[-80,60]],

  // Canadian Arctic mainland coast
  [[-88,56],[-90,58],[-92,60],[-92,62],[-94,64],[-96,64],[-98,65],[-100,66],[-102,67],[-105,68],[-108,68],[-110,68],[-112,69],[-115,70],[-118,70],[-120,71],[-125,72],[-130,72],[-135,72],[-138,71],[-140,70],[-141,70]],

  // Canadian Arctic Islands — Baffin Island
  [[-62,64],[-64,66],[-66,68],[-68,70],[-70,72],[-73,73],[-76,74],[-80,74],[-82,73],[-80,72],[-78,70],[-75,68],[-72,67],[-70,66],[-68,64],[-65,63],[-62,64]],

  // Ellesmere & Devon Islands
  [[-80,76],[-78,78],[-74,80],[-70,82],[-66,82],[-62,80],[-64,78],[-68,77],[-72,76],[-76,76],[-80,76]],

  // Victoria Island
  [[-105,70],[-108,72],[-112,73],[-115,72],[-114,70],[-110,69],[-107,70],[-105,70]],

  // Banks Island
  [[-118,72],[-122,74],[-126,74],[-124,72],[-120,71],[-118,72]],

  // Greenland
  [[-52,60],[-50,61],[-48,62],[-46,63],[-44,64],[-42,65],[-40,66],[-38,67],[-35,68],[-32,69],[-28,70],[-24,71],[-22,72],[-20,74],[-19,76],[-20,78],[-22,80],[-26,82],[-30,83],[-35,83],[-40,82],[-44,81],[-48,80],[-50,78],[-52,76],[-54,74],[-55,72],[-54,70],[-53,68],[-52,66],[-51,64],[-51,62],[-52,60]],

  // Cuba
  [[-85,22],[-84,23],[-83,23],[-82,23],[-81,23],[-80,23],[-79,23],[-78,22],[-77,21],[-76,20],[-75,20],[-74,20],[-75,20],[-77,21],[-79,22],[-81,23],[-83,23],[-85,22]],

  // Hispaniola (Haiti & Dominican Republic)
  [[-74,20],[-73,20],[-72,19],[-71,19],[-70,19],[-69,19],[-68,19],[-69,19],[-70,20],[-72,20],[-74,20]],

  // Puerto Rico
  [[-67,18],[-66,18],[-65,18],[-66,18],[-67,18]],

  // Jamaica
  [[-78,18],[-77,18],[-76,18],[-77,18],[-78,18]],

  // Lesser Antilles chain (simplified)
  [[-62,17],[-61,16],[-61,15],[-61,14],[-61,13],[-61,12]],

  // ── EUROPE ────────────────────────

  // Iberian Peninsula
  [[-10,36],[-9,37],[-9,38],[-9,39],[-9,40],[-8,41],[-8,42],[-8,43],[-7,43],[-6,44],[-5,44],[-4,44],[-3,44],[-2,44],[-1,43],[0,42],[1,41]],

  // Mediterranean Spain & France
  [[1,41],[2,42],[3,43],[4,43],[5,43],[6,43],[7,44],[8,44]],

  // French Atlantic coast
  [[-1,43],[-1,44],[-1,45],[-1,46],[-2,47],[-3,48],[-4,48],[-5,48]],

  // Brittany to English Channel
  [[-5,48],[-4,49],[-3,49],[-2,49],[-1,50],[0,50],[1,51],[2,51]],

  // North Sea coast (France/Belgium/Netherlands/Germany/Denmark)
  [[2,51],[3,51],[4,52],[5,53],[6,54],[7,55],[8,55],[8,56],[9,55],[10,55],[10,54],[11,54],[12,55],[12,56]],

  // Denmark Jutland
  [[9,55],[9,57],[10,57],[10,58],[11,57],[12,56]],

  // Great Britain
  [[-6,50],[-5,50],[-4,50],[-3,50],[-2,51],[-1,51],[0,51],[1,52],[1,53],[0,54],[-1,55],[-2,55],[-3,56],[-4,57],[-5,58],[-5,57],[-4,56],[-3,55],[-3,54],[-4,53],[-4,52],[-5,51],[-5,50],[-6,50]],

  // Scotland north coast
  [[-5,58],[-4,58],[-3,59],[-2,58],[-1,58],[0,57],[0,56]],

  // Ireland
  [[-10,52],[-10,53],[-9,54],[-8,55],[-7,55],[-6,55],[-6,54],[-6,53],[-7,52],[-8,52],[-9,52],[-10,52]],

  // Norway
  [[5,58],[5,59],[5,60],[6,61],[6,62],[7,63],[8,63],[10,63],[12,64],[13,65],[14,66],[15,67],[16,68],[16,69],[17,70],[18,70],[20,70],[22,70],[24,71],[26,71],[28,71],[30,71],[32,70]],

  // Sweden east coast & Finland
  [[18,60],[18,62],[18,64],[20,66],[22,68],[24,68],[24,66],[24,64],[26,63],[28,62],[28,61],[30,60]],

  // Baltic states & Russia Baltic
  [[22,60],[24,59],[26,58],[28,58],[28,60],[30,60]],

  // Italy — West coast
  [[8,44],[9,44],[10,44],[10,43],[11,43],[11,42],[12,42],[12,41],[13,41],[14,40],[15,40],[15,39],[16,38]],

  // Italy — East coast (Adriatic)
  [[13,46],[13,45],[13,44],[14,44],[14,43],[15,42],[16,42],[16,41],[17,41],[18,41],[18,40],[16,39],[16,38]],

  // Italy toe and heel
  [[16,38],[17,39],[18,40]],

  // Sicily
  [[13,38],[14,38],[15,37],[14,37],[13,37],[12,38],[13,38]],

  // Sardinia
  [[8,39],[9,39],[10,40],[10,41],[9,41],[8,40],[8,39]],

  // Corsica
  [[9,42],[9,43],[9,43],[8,42],[9,42]],

  // Croatia / Montenegro / Albania
  [[14,45],[15,45],[16,43],[17,43],[18,42],[19,42],[20,41],[20,40]],

  // Greece mainland
  [[20,40],[21,39],[22,38],[23,37],[24,36],[23,38],[22,39],[20,40]],

  // Greece — Peloponnese
  [[22,38],[22,37],[21,37],[22,36],[23,36],[23,37],[22,38]],

  // Crete
  [[24,35],[25,35],[26,35],[27,35],[26,35],[24,35]],

  // Turkey north coast (Black Sea)
  [[26,42],[28,42],[30,42],[32,42],[34,42],[36,42],[38,41],[40,41],[42,42]],

  // Turkey south coast (Mediterranean)
  [[26,36],[28,36],[30,37],[32,37],[34,37],[36,37],[36,36]],

  // Turkey west coast (Aegean)
  [[26,36],[26,38],[27,39],[26,40],[26,42]],

  // Black Sea south coast to north
  [[42,42],[42,44],[40,44],[38,46],[36,46],[34,46],[32,46],[30,46],[30,44],[28,44],[26,42]],

  // Caspian Sea
  [[48,42],[49,44],[50,45],[51,44],[52,44],[53,42],[54,40],[53,38],[52,37],[50,37],[48,38],[48,40],[48,42]],

  // ── AFRICA ────────────────────────

  // North Africa Mediterranean coast
  [[-6,35],[-5,36],[-4,36],[-2,36],[-1,36],[0,36],[2,37],[4,37],[5,37],[7,37],[8,37],[10,37],[11,36],[12,33],[13,33],[15,32],[18,32],[20,32],[22,32],[24,32],[26,31],[28,31],[30,31],[32,32],[34,32]],

  // West Africa (Morocco south to Senegal)
  [[-6,35],[-8,34],[-10,33],[-12,30],[-13,28],[-14,26],[-16,24],[-17,22],[-17,20],[-17,18],[-17,16],[-17,15],[-17,14]],

  // West Africa coast (Senegal to Nigeria)
  [[-17,14],[-16,13],[-16,12],[-15,11],[-14,10],[-13,10],[-12,8],[-11,7],[-10,7],[-8,5],[-7,5],[-6,5],[-5,5],[-4,5],[-3,5],[-2,5],[-1,5],[0,6],[1,6],[2,6],[3,6],[4,6]],

  // Gulf of Guinea
  [[4,6],[5,5],[6,4],[7,4],[8,4],[9,4],[10,4],[10,3],[10,2],[10,1],[9,0],[9,-1],[9,-2],[9,-3]],

  // West Central Africa coast
  [[9,-3],[10,-4],[11,-5],[12,-6],[12,-8],[12,-10],[12,-12],[13,-14],[13,-16],[14,-18],[15,-20],[16,-22],[16,-24],[17,-27],[17,-29],[18,-32],[18,-34]],

  // Southern Africa
  [[18,-34],[19,-34],[20,-35],[22,-34],[24,-34],[26,-34],[28,-33],[28,-32],[30,-30]],

  // East Africa south
  [[30,-30],[31,-28],[32,-26],[33,-25],[34,-24],[35,-22],[36,-20],[37,-18],[38,-16],[39,-14],[40,-12],[41,-10],[42,-8],[43,-6],[44,-4],[45,-2],[46,0],[47,2],[48,4],[49,6],[50,8],[50,10]],

  // Horn of Africa (Somalia)
  [[50,10],[50,12],[48,12],[46,12],[44,12],[43,12],[42,12],[42,10],[44,8],[46,6],[48,4]],

  // Red Sea — Egypt/Sudan coast
  [[34,32],[34,30],[33,28],[34,26],[35,24],[36,22],[38,20],[39,18],[40,16],[42,14],[43,12]],

  // Madagascar
  [[44,-12],[45,-13],[46,-15],[47,-17],[48,-19],[49,-21],[49,-23],[48,-25],[47,-26],[45,-26],[44,-24],[43,-22],[44,-18],[43,-16],[43,-14],[44,-12]],

  // ── MIDDLE EAST & ARABIAN PENINSULA ──

  // Levant coast
  [[34,32],[35,34],[36,36],[35,36]],

  // Red Sea east coast — Arabian Peninsula west
  [[36,28],[38,24],[39,22],[40,20],[42,18],[43,14],[44,12],[45,12]],

  // Arabian Peninsula south coast
  [[45,12],[46,13],[48,14],[50,16],[52,17],[54,17],[56,18],[58,22]],

  // Oman/UAE coast
  [[58,22],[56,24],[55,25],[54,25],[52,24],[51,24],[50,26],[49,27],[48,28],[48,30]],

  // Persian Gulf (both coasts)
  [[48,30],[49,30],[50,30],[52,28],[54,27],[55,26],[56,24]],
  [[48,30],[48,29],[48,28],[50,27],[51,26],[50,26]],

  // Iran south coast
  [[56,24],[57,26],[58,26],[60,26],[62,25],[64,25],[66,25],[68,24],[70,22]],

  // ── SOUTH ASIA ────────────────────

  // India west coast
  [[70,22],[72,21],[72,20],[72,18],[73,16],[74,14],[75,12],[76,10],[77,8]],

  // India south tip & east coast
  [[77,8],[78,8],[79,8],[80,9],[80,10],[80,12],[80,14],[81,15],[82,16],[83,17],[84,18],[85,19],[86,20],[87,21],[88,22],[89,22],[90,22]],

  // Sri Lanka
  [[80,10],[81,9],[82,8],[82,7],[81,6],[80,6],[80,7],[79,8],[80,10]],

  // ── SOUTHEAST ASIA ────────────────

  // Bangladesh / Myanmar coast
  [[90,22],[91,22],[92,21],[93,20],[94,18],[95,17],[96,16],[97,16],[98,16]],

  // Thailand / Malay Peninsula west
  [[98,16],[98,14],[98,12],[98,10],[99,8],[100,6],[100,4],[101,3],[102,2],[103,1],[104,1]],

  // Thailand Gulf / Vietnam
  [[98,16],[100,14],[100,12],[102,10],[104,8],[106,6],[106,8],[106,10],[108,12],[108,14],[108,16],[108,18],[106,20],[106,22],[108,22]],

  // China coast
  [[108,22],[110,21],[112,22],[114,22],[115,23],[117,24],[118,26],[120,28],[122,30],[122,32],[120,34],[120,36],[119,37],[118,38],[120,38],[121,39],[122,40]],

  // China north coast / Korea bay
  [[122,40],[121,40],[120,39],[118,38],[116,38],[118,40],[120,40],[121,40],[122,40],[124,40],[126,38]],

  // Korean Peninsula west & south
  [[126,38],[126,36],[126,34],[127,34],[128,35],[129,35]],

  // Korean Peninsula east
  [[129,35],[129,36],[129,37],[128,38],[129,39],[130,38],[130,36],[129,35]],

  // Japan — Kyushu
  [[130,31],[131,32],[131,33],[130,34],[131,34],[132,34]],

  // Japan — Honshu south to north
  [[132,34],[134,34],[135,34],[136,35],[137,35],[138,35],[139,36],[140,36],[140,38],[140,40],[141,42],[142,43],[143,44],[145,44]],

  // Japan — Hokkaido
  [[145,44],[145,43],[144,42],[143,42],[142,42],[141,43],[140,43],[140,44],[141,44],[142,44],[143,44]],

  // Japan — Pacific coast return
  [[145,44],[144,43],[143,42],[142,40],[140,38],[139,36],[138,35],[136,34],[134,33],[132,33],[131,32],[130,31]],

  // Japan — Shikoku
  [[133,33],[134,34],[135,34],[134,33],[133,33]],

  // Taiwan
  [[120,22],[121,23],[122,25],[121,25],[120,24],[120,22]],

  // Philippines — Luzon
  [[120,19],[121,19],[122,18],[122,16],[122,14],[121,14],[120,15],[120,17],[120,19]],

  // Philippines — Visayas / Mindanao
  [[122,14],[124,12],[126,10],[127,8],[126,7],[125,6],[124,7],[122,8],[122,10],[122,12],[122,14]],

  // Borneo
  [[109,1],[110,2],[111,2],[112,2],[113,3],[114,4],[115,5],[116,6],[117,7],[118,7],[119,7],[118,6],[118,4],[117,3],[116,1],[115,0],[114,-1],[113,-2],[112,-3],[111,-3],[110,-2],[110,-1],[109,0],[109,1]],

  // Sumatra
  [[95,6],[96,5],[97,4],[98,3],[99,2],[100,1],[101,0],[102,-1],[103,-2],[104,-3],[105,-5],[106,-6],[105,-6],[104,-5],[103,-4],[102,-3],[101,-2],[100,-1],[99,0],[98,1],[97,2],[96,3],[95,4],[95,6]],

  // Java
  [[105,-6],[106,-6],[107,-7],[108,-7],[109,-7],[110,-7],[111,-7],[112,-7],[113,-8],[114,-8],[116,-8],[114,-8],[113,-8],[112,-8],[111,-8],[110,-8],[109,-8],[108,-8],[107,-7],[106,-7],[105,-6]],

  // Sulawesi
  [[119,-4],[120,-3],[121,-2],[122,-1],[123,-1],[124,-2],[124,-3],[122,-4],[121,-5],[120,-6],[119,-5],[119,-4]],

  // Timor
  [[124,-9],[126,-9],[127,-9],[126,-9],[124,-9]],

  // Papua / New Guinea
  [[131,-1],[132,-2],[134,-3],[136,-4],[138,-4],[140,-3],[141,-3],[142,-4],[143,-4],[145,-5],[147,-6],[149,-6],[150,-6],[152,-5],[152,-4],[150,-3],[148,-2],[146,-2],[144,-2],[143,-3],[142,-3],[140,-2],[138,-1],[136,-1],[134,-1],[132,0],[131,-1]],

  // ── RUSSIA (East) ─────────────────

  // Russia Arctic coast (Kola to Urals)
  [[32,70],[34,69],[36,68],[38,68],[40,68],[42,67],[44,68],[48,68],[50,68],[55,68],[58,68],[60,68]],

  // Russia Arctic coast (Urals to Lena)
  [[60,68],[62,68],[65,69],[68,70],[70,70],[72,71],[75,72],[78,72],[80,72],[82,72],[85,73],[88,73],[90,73],[95,72],[100,72],[105,72],[110,72],[115,73],[120,72]],

  // Russia Arctic coast (Lena to Bering)
  [[120,72],[122,70],[125,68],[127,65],[128,62],[130,60],[132,58],[135,56],[136,52],[137,48],[138,46],[140,44]],

  // Russia Pacific coast
  [[140,44],[142,44],[145,44],[148,46],[150,50],[152,54],[155,58],[158,60],[160,62],[162,64],[165,65],[168,65],[170,64],[172,64],[175,65],[180,65]],

  // Kamchatka Peninsula
  [[155,58],[157,56],[159,54],[160,52],[162,52],[162,55],[160,58],[158,60]],

  // Sakhalin Island
  [[142,46],[143,48],[143,50],[144,52],[143,54],[142,52],[141,50],[142,48],[142,46]],

  // ── AUSTRALIA & OCEANIA ───────────

  // Australia — full outline
  [[115,-34],[114,-33],[114,-30],[114,-28],[114,-26],[114,-24],[115,-22],[116,-21],[118,-20],[120,-18],[122,-17],[124,-16],[126,-14],[128,-14],[130,-13],[131,-12],[133,-12],[135,-12],[136,-12],[137,-14],[137,-16],[138,-16],[139,-17],[140,-18],[141,-17],[142,-15],[142,-12],[143,-12],[144,-14],[145,-16],[146,-18],[147,-20],[148,-21],[150,-23],[151,-25],[153,-26],[153,-28],[153,-30],[152,-32],[151,-34],[150,-36],[148,-37],[147,-38],[146,-39],[144,-38],[142,-38],[141,-38],[140,-37],[139,-36],[138,-35],[137,-35],[136,-34],[134,-34],[132,-33],[130,-32],[128,-33],[126,-34],[124,-34],[122,-34],[120,-34],[118,-35],[116,-34],[115,-34]],

  // Australia — Gulf of Carpentaria detail
  [[136,-12],[136,-14],[137,-16]],
  [[132,-12],[133,-12]],

  // Tasmania
  [[145,-40],[146,-41],[148,-42],[148,-43],[147,-44],[146,-44],[145,-43],[144,-42],[145,-40]],

  // New Zealand North Island
  [[173,-35],[174,-36],[175,-37],[176,-38],[177,-39],[178,-40],[178,-42],[177,-41],[176,-40],[175,-39],[174,-38],[173,-37],[173,-35]],

  // New Zealand South Island
  [[172,-42],[173,-43],[172,-44],[171,-45],[170,-46],[168,-46],[167,-45],[167,-44],[168,-44],[170,-43],[171,-42],[172,-42]],

  // ── ISLANDS ───────────────────────

  // Iceland
  [[-24,64],[-22,65],[-20,66],[-18,66],[-16,66],[-15,65],[-14,65],[-14,64],[-16,63],[-18,63],[-20,63],[-22,64],[-24,64]],

  // Svalbard
  [[12,77],[14,78],[16,79],[18,80],[20,80],[18,79],[16,78],[14,77],[12,77]],

  // Novaya Zemlya
  [[50,72],[52,74],[54,76],[56,76],[55,74],[53,72],[50,72]],

  // Hawaii (Big Island)
  [[-156,20],[-155,20],[-155,19],[-156,19],[-156,20]],

  // New Caledonia
  [[164,-20],[166,-21],[167,-22],[166,-22],[164,-21],[164,-20]],

  // Fiji
  [[177,-17],[178,-18],[178,-19],[177,-18],[177,-17]],
];

// ═══════════════════════════════════════
// REGION & OCEAN LABELS
// ═══════════════════════════════════════

interface MapLabel {
  lon: number;
  lat: number;
  name: string;
  type: "continent" | "ocean" | "region" | "country";
}

const MAP_LABELS: MapLabel[] = [
  // Continents
  { lon: -100, lat: 48, name: "NORTH AMERICA", type: "continent" },
  { lon: -58, lat: -15, name: "SOUTH AMERICA", type: "continent" },
  { lon: 20, lat: 5, name: "AFRICA", type: "continent" },
  { lon: 15, lat: 52, name: "EUROPE", type: "continent" },
  { lon: 80, lat: 50, name: "ASIA", type: "continent" },
  { lon: 134, lat: -25, name: "AUSTRALIA", type: "continent" },
  { lon: 0, lat: -80, name: "ANTARCTICA", type: "continent" },

  // Oceans
  { lon: -150, lat: 25, name: "PACIFIC OCEAN", type: "ocean" },
  { lon: 170, lat: -30, name: "PACIFIC OCEAN", type: "ocean" },
  { lon: -35, lat: 25, name: "ATLANTIC OCEAN", type: "ocean" },
  { lon: -35, lat: -30, name: "S. ATLANTIC", type: "ocean" },
  { lon: 75, lat: -25, name: "INDIAN OCEAN", type: "ocean" },
  { lon: 0, lat: 78, name: "ARCTIC OCEAN", type: "ocean" },
  { lon: -150, lat: -50, name: "SOUTHERN OCEAN", type: "ocean" },

  // Regions & Countries
  { lon: -98, lat: 38, name: "USA", type: "country" },
  { lon: -105, lat: 58, name: "CANADA", type: "country" },
  { lon: -103, lat: 24, name: "MEXICO", type: "country" },
  { lon: -85, lat: 14, name: "CENTRAL AM.", type: "region" },
  { lon: -75, lat: -5, name: "PERU", type: "country" },
  { lon: -52, lat: -8, name: "BRAZIL", type: "country" },
  { lon: -64, lat: -35, name: "ARGENTINA", type: "country" },
  { lon: -70, lat: -22, name: "BOLIVIA", type: "country" },
  { lon: -68, lat: 5, name: "COLOMBIA", type: "country" },
  { lon: -66, lat: 8, name: "VENEZUELA", type: "country" },
  { lon: -72, lat: -32, name: "CHILE", type: "country" },
  { lon: -42, lat: 72, name: "GREENLAND", type: "region" },
  { lon: -20, lat: 65, name: "ICELAND", type: "country" },
  { lon: -4, lat: 40, name: "SPAIN", type: "country" },
  { lon: 2, lat: 47, name: "FRANCE", type: "country" },
  { lon: -4, lat: 54, name: "UK", type: "country" },
  { lon: -8, lat: 53, name: "IRELAND", type: "country" },
  { lon: 10, lat: 51, name: "GERMANY", type: "country" },
  { lon: 12, lat: 43, name: "ITALY", type: "country" },
  { lon: 20, lat: 52, name: "POLAND", type: "country" },
  { lon: 25, lat: 47, name: "ROMANIA", type: "country" },
  { lon: 24, lat: 38, name: "GREECE", type: "country" },
  { lon: 35, lat: 39, name: "TURKEY", type: "country" },
  { lon: 15, lat: 63, name: "NORWAY", type: "country" },
  { lon: 18, lat: 60, name: "SWEDEN", type: "country" },
  { lon: 26, lat: 62, name: "FINLAND", type: "country" },
  { lon: 37, lat: 56, name: "RUSSIA", type: "country" },
  { lon: 30, lat: 50, name: "UKRAINE", type: "country" },
  { lon: 0, lat: 32, name: "MOROCCO", type: "country" },
  { lon: 8, lat: 30, name: "ALGERIA", type: "country" },
  { lon: 18, lat: 28, name: "LIBYA", type: "country" },
  { lon: 30, lat: 27, name: "EGYPT", type: "country" },
  { lon: 45, lat: 10, name: "SOMALIA", type: "country" },
  { lon: 38, lat: 8, name: "ETHIOPIA", type: "country" },
  { lon: 35, lat: -5, name: "TANZANIA", type: "country" },
  { lon: 25, lat: -3, name: "CONGO", type: "country" },
  { lon: -5, lat: 8, name: "W. AFRICA", type: "region" },
  { lon: 25, lat: -28, name: "S. AFRICA", type: "country" },
  { lon: 47, lat: -19, name: "MADAGASCAR", type: "country" },
  { lon: 45, lat: 24, name: "SAUDI ARABIA", type: "country" },
  { lon: 55, lat: 23, name: "UAE", type: "country" },
  { lon: 53, lat: 32, name: "IRAN", type: "country" },
  { lon: 44, lat: 34, name: "IRAQ", type: "country" },
  { lon: 68, lat: 30, name: "PAKISTAN", type: "country" },
  { lon: 80, lat: 22, name: "INDIA", type: "country" },
  { lon: 81, lat: 7, name: "SRI LANKA", type: "country" },
  { lon: 90, lat: 24, name: "BANGLADESH", type: "country" },
  { lon: 96, lat: 20, name: "MYANMAR", type: "country" },
  { lon: 101, lat: 15, name: "THAILAND", type: "country" },
  { lon: 107, lat: 16, name: "VIETNAM", type: "country" },
  { lon: 104, lat: 5, name: "MALAYSIA", type: "country" },
  { lon: 115, lat: 1, name: "BORNEO", type: "region" },
  { lon: 101, lat: -1, name: "SUMATRA", type: "region" },
  { lon: 110, lat: -7, name: "JAVA", type: "region" },
  { lon: 121, lat: -3, name: "SULAWESI", type: "region" },
  { lon: 138, lat: -4, name: "PAPUA", type: "region" },
  { lon: 122, lat: 14, name: "PHILIPPINES", type: "country" },
  { lon: 121, lat: 24, name: "TAIWAN", type: "country" },
  { lon: 105, lat: 35, name: "CHINA", type: "country" },
  { lon: 128, lat: 37, name: "KOREA", type: "country" },
  { lon: 138, lat: 37, name: "JAPAN", type: "country" },
  { lon: 90, lat: 48, name: "MONGOLIA", type: "country" },
  { lon: 68, lat: 42, name: "KAZAKHSTAN", type: "country" },
  { lon: 155, lat: 56, name: "KAMCHATKA", type: "region" },
  { lon: 135, lat: 62, name: "SIBERIA", type: "region" },
  { lon: 175, lat: -40, name: "NEW ZEALAND", type: "country" },
  { lon: -75, lat: 20, name: "CARIBBEAN", type: "region" },
  { lon: 100, lat: -2, name: "INDONESIA", type: "country" },

  // Seas
  { lon: 35, lat: 35, name: "MEDITERRANEAN", type: "ocean" },
  { lon: 50, lat: 30, name: "PERSIAN GULF", type: "ocean" },
  { lon: 40, lat: 20, name: "RED SEA", type: "ocean" },
  { lon: 35, lat: 44, name: "BLACK SEA", type: "ocean" },
  { lon: 50, lat: 42, name: "CASPIAN", type: "ocean" },
  { lon: 88, lat: 14, name: "BAY OF BENGAL", type: "ocean" },
  { lon: 110, lat: 14, name: "S. CHINA SEA", type: "ocean" },
  { lon: 140, lat: 30, name: "PACIFIC", type: "ocean" },
  { lon: -80, lat: 25, name: "GULF OF MEXICO", type: "ocean" },
  { lon: 18, lat: 58, name: "BALTIC", type: "ocean" },
  { lon: 4, lat: 56, name: "NORTH SEA", type: "ocean" },
  { lon: -70, lat: 60, name: "HUDSON BAY", type: "ocean" },
  { lon: -170, lat: 60, name: "BERING SEA", type: "ocean" },
  { lon: 140, lat: 55, name: "SEA OF OKHOTSK", type: "ocean" },
  { lon: 60, lat: -40, name: "INDIAN OCEAN", type: "ocean" },
  { lon: 135, lat: -10, name: "ARAFURA SEA", type: "ocean" },
  { lon: 155, lat: -30, name: "TASMAN SEA", type: "ocean" },
];

// ═══════════════════════════════════════
// MAJOR CITIES
// ═══════════════════════════════════════

const CITIES: [number, number, string][] = [
  [40.7, -74.0, "New York"], [51.5, -0.1, "London"], [35.7, 139.7, "Tokyo"],
  [22.3, 114.2, "Hong Kong"], [37.8, -122.4, "San Francisco"], [-33.9, 151.2, "Sydney"],
  [55.8, 37.6, "Moscow"], [1.3, 103.8, "Singapore"], [48.9, 2.4, "Paris"],
  [52.5, 13.4, "Berlin"], [19.1, 72.9, "Mumbai"], [-23.6, -46.6, "São Paulo"],
  [39.9, 116.4, "Beijing"], [37.6, 127.0, "Seoul"], [25.0, 55.3, "Dubai"],
  [30.0, 31.2, "Cairo"], [-1.3, 36.8, "Nairobi"], [33.9, -118.2, "Los Angeles"],
  [41.9, -87.6, "Chicago"], [49.3, -123.1, "Vancouver"], [59.3, 18.1, "Stockholm"],
  [28.6, 77.2, "Delhi"], [31.2, 121.5, "Shanghai"], [13.8, 100.5, "Bangkok"],
  [-34.6, -58.4, "Buenos Aires"], [14.6, 121.0, "Manila"], [64.1, -21.9, "Reykjavik"],
  [-6.2, 106.8, "Jakarta"], [-37.8, 144.9, "Melbourne"], [43.7, -79.4, "Toronto"],
  [35.2, -80.8, "Charlotte"], [29.8, -95.4, "Houston"], [25.8, -80.2, "Miami"],
  [47.6, -122.3, "Seattle"], [38.9, -77.0, "Washington"], [42.4, -71.1, "Boston"],
  [45.5, -73.6, "Montreal"], [53.5, -6.3, "Dublin"], [41.0, 29.0, "Istanbul"],
  [23.1, 113.3, "Guangzhou"], [-22.9, -43.2, "Rio de Janeiro"], [6.5, 3.4, "Lagos"],
  [-33.9, 18.4, "Cape Town"], [35.7, 51.4, "Tehran"], [24.7, 46.7, "Riyadh"],
  [56.9, 24.1, "Riga"], [59.4, 24.7, "Tallinn"], [60.2, 25.0, "Helsinki"],
  [50.1, 14.4, "Prague"], [47.5, 19.0, "Budapest"], [44.4, 26.1, "Bucharest"],
  [40.4, -3.7, "Madrid"], [41.4, 2.2, "Barcelona"], [45.5, 9.2, "Milan"],
  [41.9, 12.5, "Rome"], [37.0, -122.1, "Silicon Valley"],
];

// ═══════════════════════════════════════
// COMPONENT
// ═══════════════════════════════════════

export default function MonitorDashboard(props: Props) {
  let canvasRef: HTMLCanvasElement | undefined;
  let animFrame: number;
  let dataInterval: number;

  const [iss, setISS] = createSignal<ISSPos | null>(null);
  const [news, setNews] = createSignal<NewsItem[]>([]);
  const [stats, setStats] = createSignal<SysStats | null>(null);
  const [activity, setActivity] = createSignal<Activity[]>([]);
  const [publicIp, setPublicIp] = createSignal("...");
  const [utc, setUtc] = createSignal("");
  const [tickerOffset, setTickerOffset] = createSignal(0);

  // Animation state (not reactive)
  let radarAngle = 0;
  let issTrail: { x: number; y: number; age: number }[] = [];
  let connectionLines: { x1: number; y1: number; x2: number; y2: number; progress: number; speed: number }[] = [];
  let pulsingDots: { x: number; y: number; phase: number; speed: number }[] = [];
  let frame = 0;

  onMount(async () => {
    fetchAll();
    dataInterval = window.setInterval(fetchAll, 15000);
    const issTimer = window.setInterval(fetchISS, 5000);

    const clockTimer = window.setInterval(() => {
      setUtc(new Date().toISOString().slice(11, 19));
    }, 1000);
    setUtc(new Date().toISOString().slice(11, 19));

    const tickerTimer = window.setInterval(() => {
      setTickerOffset((o) => o + 1);
    }, 50);

    if (canvasRef) {
      initCanvas();
      startRenderLoop();
    }

    onCleanup(() => {
      cancelAnimationFrame(animFrame);
      clearInterval(dataInterval);
      clearInterval(issTimer);
      clearInterval(clockTimer);
      clearInterval(tickerTimer);
    });
  });

  async function fetchAll() {
    fetchISS();
    try { const n = (await invoke("monitor_news")) as NewsItem[]; setNews(n); } catch (_) {}
    try { const s = (await invoke("monitor_system_stats")) as SysStats; setStats(s); } catch (_) {}
    try { const a = (await invoke("monitor_activity")) as Activity[]; setActivity(a); } catch (_) {}
    try { const ip = (await invoke("monitor_public_ip")) as string; setPublicIp(ip); } catch (_) {}
  }

  async function fetchISS() {
    try { const pos = (await invoke("monitor_iss_position")) as ISSPos; setISS(pos); } catch (_) {}
  }

  function initCanvas() {
    const resize = () => {
      if (!canvasRef) return;
      const rect = canvasRef.parentElement!.getBoundingClientRect();
      canvasRef.width = rect.width * window.devicePixelRatio;
      canvasRef.height = rect.height * window.devicePixelRatio;
      canvasRef.style.width = rect.width + "px";
      canvasRef.style.height = rect.height + "px";
      initConnections();
      initPulsingDots();
    };
    resize();
    window.addEventListener("resize", resize);
    onCleanup(() => window.removeEventListener("resize", resize));
  }

  function initConnections() {
    connectionLines = [];
    for (let i = 0; i < 8; i++) {
      const a = CITIES[Math.floor(Math.random() * CITIES.length)];
      let b = CITIES[Math.floor(Math.random() * CITIES.length)];
      while (b === a) b = CITIES[Math.floor(Math.random() * CITIES.length)];
      const pa = geoToCanvas(a[0], a[1]);
      const pb = geoToCanvas(b[0], b[1]);
      connectionLines.push({
        x1: pa.x, y1: pa.y, x2: pb.x, y2: pb.y,
        progress: Math.random(),
        speed: 0.002 + Math.random() * 0.005,
      });
    }
  }

  function initPulsingDots() {
    pulsingDots = CITIES.map(([lat, lon]) => {
      const p = geoToCanvas(lat, lon);
      return { x: p.x, y: p.y, phase: Math.random() * Math.PI * 2, speed: 0.02 + Math.random() * 0.04 };
    });
  }

  function geoToCanvas(lat: number, lon: number) {
    if (!canvasRef) return { x: 0, y: 0 };
    const w = canvasRef.width;
    const h = canvasRef.height;
    const x = ((lon + 180) / 360) * w;
    const y = ((90 - lat) / 180) * h;
    return { x, y };
  }

  // ─── Render Loop ──────────────────

  function startRenderLoop() {
    function render() {
      if (!canvasRef) return;
      const ctx = canvasRef.getContext("2d")!;
      const w = canvasRef.width;
      const h = canvasRef.height;
      const dpr = window.devicePixelRatio;
      frame++;

      ctx.clearRect(0, 0, w, h);

      // ── Background grid ──
      ctx.strokeStyle = "rgba(0,255,65,0.04)";
      ctx.lineWidth = dpr;
      const gridStep = 40 * dpr;
      for (let x = 0; x < w; x += gridStep) {
        ctx.beginPath(); ctx.moveTo(x, 0); ctx.lineTo(x, h); ctx.stroke();
      }
      for (let y = 0; y < h; y += gridStep) {
        ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(w, y); ctx.stroke();
      }

      // ── Latitude / Longitude lines with labels ──
      ctx.font = `${8 * dpr}px "JetBrains Mono", monospace`;
      ctx.textAlign = "left";

      // Major latitude lines
      const latLines = [-60, -40, -20, 0, 20, 40, 60, 80];
      for (const lat of latLines) {
        const p = geoToCanvas(lat, -180);
        const isEquator = lat === 0;
        ctx.strokeStyle = isEquator ? "rgba(0,255,65,0.12)" : "rgba(0,255,65,0.06)";
        ctx.lineWidth = isEquator ? 1.5 * dpr : 0.5 * dpr;
        ctx.setLineDash(isEquator ? [] : [4 * dpr, 4 * dpr]);
        ctx.beginPath(); ctx.moveTo(0, p.y); ctx.lineTo(w, p.y); ctx.stroke();
        ctx.setLineDash([]);
        ctx.fillStyle = "rgba(0,255,65,0.25)";
        ctx.fillText(`${lat}°`, 4 * dpr, p.y - 2 * dpr);
      }

      // Major longitude lines
      const lonLines = [-150, -120, -90, -60, -30, 0, 30, 60, 90, 120, 150, 180];
      for (const lon of lonLines) {
        const p = geoToCanvas(0, lon);
        const isPM = lon === 0;
        ctx.strokeStyle = isPM ? "rgba(0,255,65,0.12)" : "rgba(0,255,65,0.06)";
        ctx.lineWidth = isPM ? 1.5 * dpr : 0.5 * dpr;
        ctx.setLineDash(isPM ? [] : [4 * dpr, 4 * dpr]);
        ctx.beginPath(); ctx.moveTo(p.x, 0); ctx.lineTo(p.x, h); ctx.stroke();
        ctx.setLineDash([]);
        ctx.fillStyle = "rgba(0,255,65,0.25)";
        ctx.fillText(`${lon}°`, p.x + 2 * dpr, 10 * dpr);
      }

      // Tropics & Arctic/Antarctic circles
      const specialLats = [
        { lat: 23.44, label: "TROPIC OF CANCER" },
        { lat: -23.44, label: "TROPIC OF CAPRICORN" },
        { lat: 66.56, label: "ARCTIC CIRCLE" },
        { lat: -66.56, label: "ANTARCTIC CIRCLE" },
      ];
      for (const { lat, label } of specialLats) {
        const p = geoToCanvas(lat, -180);
        ctx.strokeStyle = "rgba(0,255,65,0.05)";
        ctx.lineWidth = 0.5 * dpr;
        ctx.setLineDash([2 * dpr, 6 * dpr]);
        ctx.beginPath(); ctx.moveTo(0, p.y); ctx.lineTo(w, p.y); ctx.stroke();
        ctx.setLineDash([]);
        ctx.fillStyle = "rgba(0,255,65,0.12)";
        ctx.font = `${6 * dpr}px "JetBrains Mono", monospace`;
        ctx.fillText(label, w - ctx.measureText(label).width - 6 * dpr, p.y - 2 * dpr);
      }

      // ── Draw coastlines ──
      ctx.strokeStyle = "rgba(0,255,65,0.35)";
      ctx.lineWidth = 1.2 * dpr;
      ctx.lineJoin = "round";
      ctx.lineCap = "round";
      for (const path of COASTS) {
        ctx.beginPath();
        for (let i = 0; i < path.length; i++) {
          const p = geoToCanvas(path[i][1], path[i][0]);
          if (i === 0) ctx.moveTo(p.x, p.y);
          else ctx.lineTo(p.x, p.y);
        }
        ctx.stroke();
      }

      // Coastline vertex dots
      ctx.fillStyle = "rgba(0,255,65,0.2)";
      for (const path of COASTS) {
        for (const [lon, lat] of path) {
          const p = geoToCanvas(lat, lon);
          ctx.beginPath(); ctx.arc(p.x, p.y, 0.8 * dpr, 0, Math.PI * 2); ctx.fill();
        }
      }

      // ── Region & ocean labels ──
      ctx.textAlign = "center";
      for (const lbl of MAP_LABELS) {
        const p = geoToCanvas(lbl.lat, lbl.lon);
        switch (lbl.type) {
          case "continent":
            ctx.fillStyle = "rgba(0,255,65,0.10)";
            ctx.font = `bold ${12 * dpr}px "JetBrains Mono", monospace`;
            ctx.fillText(lbl.name, p.x, p.y);
            break;
          case "ocean":
            ctx.fillStyle = "rgba(0,120,255,0.08)";
            ctx.font = `italic ${9 * dpr}px "JetBrains Mono", monospace`;
            ctx.fillText(lbl.name, p.x, p.y);
            break;
          case "country":
            ctx.fillStyle = "rgba(0,255,65,0.18)";
            ctx.font = `${7 * dpr}px "JetBrains Mono", monospace`;
            ctx.fillText(lbl.name, p.x, p.y);
            break;
          case "region":
            ctx.fillStyle = "rgba(0,255,65,0.14)";
            ctx.font = `${7 * dpr}px "JetBrains Mono", monospace`;
            ctx.fillText(lbl.name, p.x, p.y);
            break;
        }
      }
      ctx.textAlign = "left";

      // ── Pulsing city dots ──
      for (const dot of pulsingDots) {
        dot.phase += dot.speed;
        const pulse = 0.3 + Math.sin(dot.phase) * 0.7;
        const r = (1.5 + pulse * 2.5) * dpr;
        ctx.fillStyle = `rgba(0,255,65,${0.15 + pulse * 0.45})`;
        ctx.shadowColor = "#00ff41";
        ctx.shadowBlur = 5 * dpr;
        ctx.beginPath(); ctx.arc(dot.x, dot.y, r, 0, Math.PI * 2); ctx.fill();
      }
      ctx.shadowBlur = 0;

      // ── Connection lines ──
      for (const line of connectionLines) {
        line.progress += line.speed;
        if (line.progress > 1) {
          line.progress = 0;
          const a = CITIES[Math.floor(Math.random() * CITIES.length)];
          let b = CITIES[Math.floor(Math.random() * CITIES.length)];
          while (b === a) b = CITIES[Math.floor(Math.random() * CITIES.length)];
          const pa = geoToCanvas(a[0], a[1]);
          const pb = geoToCanvas(b[0], b[1]);
          line.x1 = pa.x; line.y1 = pa.y; line.x2 = pb.x; line.y2 = pb.y;
          line.speed = 0.002 + Math.random() * 0.005;
        }

        const grad = ctx.createLinearGradient(line.x1, line.y1, line.x2, line.y2);
        const p = line.progress;
        grad.addColorStop(Math.max(0, p - 0.15), "transparent");
        grad.addColorStop(p, "rgba(0,255,65,0.6)");
        grad.addColorStop(Math.min(1, p + 0.02), "transparent");
        ctx.strokeStyle = grad;
        ctx.lineWidth = 1 * dpr;
        ctx.beginPath();
        ctx.moveTo(line.x1, line.y1);
        ctx.lineTo(line.x2, line.y2);
        ctx.stroke();

        // Moving dot
        const dx = line.x1 + (line.x2 - line.x1) * p;
        const dy = line.y1 + (line.y2 - line.y1) * p;
        ctx.fillStyle = "#00ff41";
        ctx.shadowColor = "#00ff41";
        ctx.shadowBlur = 8 * dpr;
        ctx.beginPath(); ctx.arc(dx, dy, 2 * dpr, 0, Math.PI * 2); ctx.fill();
        ctx.shadowBlur = 0;
      }

      // ── ISS Position ──
      const issData = iss();
      if (issData) {
        const ip = geoToCanvas(issData.latitude, issData.longitude);
        issTrail.push({ x: ip.x, y: ip.y, age: 0 });
        if (issTrail.length > 100) issTrail.shift();

        // Trail
        for (let i = 0; i < issTrail.length; i++) {
          const t = issTrail[i];
          t.age++;
          const a = 1 - t.age / 100;
          ctx.fillStyle = `rgba(255,100,100,${a * 0.5})`;
          ctx.beginPath(); ctx.arc(t.x, t.y, 1 * dpr, 0, Math.PI * 2); ctx.fill();
        }

        // ISS dot with ring
        ctx.strokeStyle = "rgba(255,68,68,0.4)";
        ctx.lineWidth = 1 * dpr;
        const issRingR = (8 + Math.sin(frame * 0.05) * 3) * dpr;
        ctx.beginPath(); ctx.arc(ip.x, ip.y, issRingR, 0, Math.PI * 2); ctx.stroke();

        ctx.fillStyle = "#ff4444";
        ctx.shadowColor = "#ff4444";
        ctx.shadowBlur = 14 * dpr;
        ctx.beginPath(); ctx.arc(ip.x, ip.y, 4 * dpr, 0, Math.PI * 2); ctx.fill();
        ctx.shadowBlur = 0;

        // Label
        ctx.fillStyle = "#ff4444";
        ctx.font = `bold ${10 * dpr}px "JetBrains Mono", monospace`;
        ctx.textAlign = "left";
        ctx.fillText("ISS", ip.x + 12 * dpr, ip.y - 8 * dpr);
        ctx.font = `${7 * dpr}px "JetBrains Mono", monospace`;
        ctx.fillStyle = "rgba(255,100,100,0.7)";
        ctx.fillText(
          `${issData.latitude.toFixed(1)}°, ${issData.longitude.toFixed(1)}°`,
          ip.x + 12 * dpr, ip.y + 2 * dpr
        );
      }

      // ── Radar sweep ──
      radarAngle += 0.008;
      const cx = w / 2;
      const cy = h / 2;
      const rr = Math.min(w, h) * 0.45;

      // Sweep line
      const sx = cx + Math.cos(radarAngle) * rr;
      const sy = cy + Math.sin(radarAngle) * rr;
      ctx.strokeStyle = "rgba(0,255,65,0.12)";
      ctx.lineWidth = 1 * dpr;
      ctx.beginPath(); ctx.moveTo(cx, cy); ctx.lineTo(sx, sy); ctx.stroke();

      // Sweep glow arc
      for (let i = 0; i < 30; i++) {
        const a = radarAngle - i * 0.02;
        const fade = 1 - i / 30;
        ctx.strokeStyle = `rgba(0,255,65,${0.04 * fade})`;
        const ex = cx + Math.cos(a) * rr;
        const ey = cy + Math.sin(a) * rr;
        ctx.beginPath(); ctx.moveTo(cx, cy); ctx.lineTo(ex, ey); ctx.stroke();
      }

      animFrame = requestAnimationFrame(render);
    }
    render();
  }

  // ─── Helpers ──────────────────────

  function tzTime(offset: number) {
    const d = new Date();
    d.setUTCHours(d.getUTCHours() + offset);
    return d.toISOString().slice(11, 16);
  }

  function formatUptime(secs: number) {
    const d = Math.floor(secs / 86400);
    const h = Math.floor((secs % 86400) / 3600);
    const m = Math.floor((secs % 3600) / 60);
    return d > 0 ? `${d}d ${h}h ${m}m` : `${h}h ${m}m`;
  }

  const tickerText = () => {
    const items = news();
    if (!items.length) return "  >>>  FLUX MONITOR — LOADING FEEDS...  <<<  ";
    return items.map((n) => `  ▸ ${n.title}  [${n.source}]  `).join("  ◈  ");
  };

  return (
    <div class="monitor-overlay" onClick={() => props.onClose()}>
      <div class="monitor-dashboard" onClick={(e) => e.stopPropagation()}>

        {/* ── Top Bar ── */}
        <div class="monitor-topbar">
          <div class="monitor-topbar-left">
            <span class="monitor-logo">⚡ FLUX MONITOR</span>
            <span class="monitor-status-dot" />
            <span class="monitor-status-text">LIVE</span>
          </div>
          <div class="monitor-topbar-center">
            <div class="monitor-clock-group">
              <div class="monitor-tz"><span class="monitor-tz-label">UTC</span><span class="monitor-tz-time">{utc()}</span></div>
              <div class="monitor-tz"><span class="monitor-tz-label">NYC</span><span class="monitor-tz-time">{tzTime(-5)}</span></div>
              <div class="monitor-tz"><span class="monitor-tz-label">LON</span><span class="monitor-tz-time">{tzTime(0)}</span></div>
              <div class="monitor-tz"><span class="monitor-tz-label">TYO</span><span class="monitor-tz-time">{tzTime(9)}</span></div>
              <div class="monitor-tz"><span class="monitor-tz-label">SYD</span><span class="monitor-tz-time">{tzTime(11)}</span></div>
            </div>
          </div>
          <div class="monitor-topbar-right">
            <span class="monitor-close" onClick={() => props.onClose()}>✕ ESC</span>
          </div>
        </div>

        {/* ── Main Content ── */}
        <div class="monitor-main">

          {/* Left Panel */}
          <div class="monitor-side monitor-left">
            <div class="monitor-panel">
              <div class="monitor-panel-title">⊞ SYSTEM</div>
              <Show when={stats()} fallback={<div class="monitor-dim">Loading...</div>}>
                <div class="monitor-kv"><span>HOST</span><span>{stats()!.hostname}</span></div>
                <div class="monitor-kv"><span>OS</span><span>{stats()!.os}</span></div>
                <div class="monitor-kv"><span>CPU</span><span>{stats()!.cpu_count} cores</span></div>
                <div class="monitor-kv"><span>MEM</span><span>{stats()!.memory_used_mb}/{stats()!.memory_total_mb} MB</span></div>
                <div class="monitor-kv"><span>UP</span><span>{formatUptime(stats()!.uptime_secs)}</span></div>
                <div class="monitor-kv"><span>LAN</span><span>{stats()!.local_ip}</span></div>
                <div class="monitor-kv"><span>WAN</span><span>{publicIp()}</span></div>
              </Show>
            </div>

            <div class="monitor-panel">
              <div class="monitor-panel-title">🛰 ISS TRACKER</div>
              <Show when={iss()} fallback={<div class="monitor-dim">Acquiring...</div>}>
                <div class="monitor-kv"><span>LAT</span><span class="monitor-accent">{iss()!.latitude.toFixed(2)}°</span></div>
                <div class="monitor-kv"><span>LON</span><span class="monitor-accent">{iss()!.longitude.toFixed(2)}°</span></div>
                <div class="monitor-kv"><span>ALT</span><span>{iss()!.altitude.toFixed(0)} km</span></div>
                <div class="monitor-kv"><span>VEL</span><span>{iss()!.velocity.toFixed(0)} km/h</span></div>
              </Show>
            </div>

            <div class="monitor-panel">
              <div class="monitor-panel-title">⚡ ACTIVITY</div>
              <div class="monitor-activity-list">
                <For each={activity().slice(0, 8)}>
                  {(a) => (
                    <div class="monitor-activity-item">
                      <span class="monitor-activity-dot" style={{ opacity: a.intensity }} />
                      <span class="monitor-activity-city">{a.label}</span>
                      <span class="monitor-activity-type">{a.event_type}</span>
                    </div>
                  )}
                </For>
              </div>
            </div>
          </div>

          {/* Center — World Map Canvas */}
          <div class="monitor-center">
            <canvas ref={canvasRef} class="monitor-canvas" />
          </div>

          {/* Right Panel */}
          <div class="monitor-side monitor-right">
            <div class="monitor-panel monitor-panel-full">
              <div class="monitor-panel-title">📡 LIVE FEED</div>
              <div class="monitor-news-list">
                <For each={news()}>
                  {(item, i) => (
                    <div class="monitor-news-item">
                      <div class="monitor-news-idx">{String(i() + 1).padStart(2, "0")}</div>
                      <div class="monitor-news-body">
                        <div class="monitor-news-title">{item.title}</div>
                        <div class="monitor-news-meta">
                          {item.source} · {item.timestamp}
                        </div>
                      </div>
                    </div>
                  )}
                </For>
                <Show when={news().length === 0}>
                  <div class="monitor-dim" style={{ padding: "12px" }}>Fetching headlines...</div>
                </Show>
              </div>
            </div>
          </div>
        </div>

        {/* ── Bottom Ticker ── */}
        <div class="monitor-ticker">
          <div
            class="monitor-ticker-text"
            style={{ transform: `translateX(-${tickerOffset() % (tickerText().length * 8)}px)` }}
          >
            {tickerText()}{tickerText()}
          </div>
        </div>

        {/* Scanline overlay */}
        <div class="monitor-scanlines" />
      </div>
    </div>
  );
}