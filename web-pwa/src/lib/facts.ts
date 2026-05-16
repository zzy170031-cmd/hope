import type { SourceFacts } from "./types";

const genericPersonTerms = new Set(["主角", "敌人", "路人", "少女", "少年", "男人", "女人", "老人"]);
const explicitRoleTerms = ["主角", "敌人", "黑衣追兵", "追兵", "守军", "将领", "使者", "指挥官", "队友"];
const locationHints = /(巷|街|城|殿|帐|台|楼|桥|港|营地|沙盘|战场|城门|废墟|宫墙|军帐|都城)/;
const eventHints = /(追|逃|守|攻|谈|救|护|逼近|包围|冲突|伏击|对话|争执|撤离|布阵|围城|推进)/;
const constraintHints = /(必须|不能|不得|限时|之前|之内|同时|仍要|只能|还要)/;
const objectHints = /(兵器|长枪|卷轴|底片|密钥|地图|战报|旗帜|面具|机甲|阵盘)/;

function unique(items: string[]): string[] {
  return Array.from(new Set(items.filter(Boolean)));
}

export function extractSourceFacts(sourceText: string): SourceFacts {
  const normalized = sourceText.replace(/\r/g, "").trim();
  const clauses = normalized
    .split(/[\n。！？；]/)
    .map((item) => item.trim())
    .filter((item) => item.length >= 3);

  const nameCandidates = normalized.match(/[\u4e00-\u9fa5]{2,4}/g) ?? [];
  const characters = unique(
    nameCandidates.filter((item) => {
      if (genericPersonTerms.has(item)) {
        return false;
      }
      if (/(场景|镜头|画面|故事|正文|分镜|提示词|规则|时长)/.test(item)) {
        return false;
      }
      return true;
    }),
  ).slice(0, 8);

  const roleMatches = explicitRoleTerms.filter((item) => normalized.includes(item));
  const locationSeeds = clauses.filter((item) => locationHints.test(item));
  const eventSeeds = clauses.filter((item) => eventHints.test(item));
  const constraintSeeds = clauses.filter((item) => constraintHints.test(item));
  const objectSeeds = clauses.filter((item) => objectHints.test(item));

  return {
    characters: unique([...characters, ...roleMatches]).slice(0, 8),
    locations: unique(locationSeeds).slice(0, 6),
    events: unique(eventSeeds).slice(0, 6),
    constraints: unique(constraintSeeds).slice(0, 6),
    visibleObjects: unique(objectSeeds).slice(0, 6),
    beats: clauses.slice(0, 8),
  };
}

export function summarizeFacts(facts: SourceFacts): string {
  return [
    facts.characters.length ? `人物：${facts.characters.join("、")}` : "",
    facts.locations.length ? `地点：${facts.locations.join("、")}` : "",
    facts.events.length ? `事件：${facts.events.join("、")}` : "",
    facts.constraints.length ? `约束：${facts.constraints.join("、")}` : "",
  ]
    .filter(Boolean)
    .join("\n");
}
