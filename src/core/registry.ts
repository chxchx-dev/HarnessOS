import type { Skill, Tool } from "./contracts.ts";

export class Registry {
  private readonly skills = new Map<string, Skill>();
  private readonly tools = new Map<string, Tool>();

  registerSkill(skill: Skill): void {
    const key = `${skill.id}@${skill.version}`;
    if (this.skills.has(key)) throw new Error(`Skill already registered: ${key}`);
    this.skills.set(key, skill);
  }
  registerTool(tool: Tool): void {
    if (this.tools.has(tool.id)) throw new Error(`Tool already registered: ${tool.id}`);
    this.tools.set(tool.id, tool);
  }
  getSkill(id: string, version: string): Skill | undefined {
    return this.skills.get(`${id}@${version}`);
  }
  getTool(id: string): Tool | undefined { return this.tools.get(id); }
  listSkills(): Array<{ id: string; version: string }> {
    return [...this.skills.values()].map(({ id, version }) => ({ id, version }));
  }
}
