import { Model } from "game_of_life_engine";
import { drawContextType } from "../../ports/drawContext";

type cbType = (param: keyof systemModelType) => void;

export type systemModelType = {
    readonly model: Model;
    readonly gap: number;
    readonly fps: number;
    readonly status: "resumed" | "paused";
    readonly dimension: number;
    readonly drawContext: drawContextType;
};

export class SystemModel {
    private readonly onChangeListeners: (cbType)[] = [];

    private model: Model;
    private gap: number;
    private fps: number;
    private status: "resumed" | "paused";
    private dimension: number;
    private drawContext: drawContextType;

    constructor(systemModel: systemModelType) {
        this.model = systemModel.model;
        this.gap = systemModel.gap;
        this.fps = systemModel.fps;
        this.status = systemModel.status;
        this.dimension = systemModel.dimension;
        this.drawContext = systemModel.drawContext;
    }

    public setModel(model: systemModelType["model"]): void {
        this.model = model;
        this.onChange("model");
    }

    public setGap(gap: systemModelType["gap"]): void {
        this.gap = gap;
        this.onChange("gap");
    }

    public setFps(fps: systemModelType["fps"]): void {
        this.fps = fps;
        this.onChange("fps");
    }

    public setStatus(status: systemModelType["status"]): void {
        this.status = status;
        this.onChange("status");
    }

    public setDimension(
        dimension: systemModelType["dimension"],
    ): void {
        this.dimension = dimension;
        this.onChange("dimension");
    }

    public setDrawContext(
        drawContext: systemModelType["drawContext"],
    ): void {
        this.drawContext = drawContext;
        this.onChange("drawContext");
    }

    public getModel(): systemModelType {
        return {
            model: this.model,
            gap: this.gap,
            fps: this.fps,
            status: this.status,
            dimension: this.dimension,
            drawContext: this.drawContext,
        };
    }

    public addOnChangeListener(cb: cbType): void {
        this.onChangeListeners.push(cb);
    }

    private onChange(param: keyof systemModelType): void {
        this.onChangeListeners.forEach((cb) => cb(param));
    }
}
