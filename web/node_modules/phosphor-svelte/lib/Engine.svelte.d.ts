import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `EngineIcon` instead.
 *
 * @example
 * ```svelte
 * <Engine color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Engine: Component<IconComponentProps, {}, "">;
type Engine = ReturnType<typeof Engine>;
export default Engine;
