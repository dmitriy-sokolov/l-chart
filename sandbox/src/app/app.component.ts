import {
  AfterViewInit,
  Component,
  ViewChild,
  ElementRef,
  OnDestroy,
  OnInit
} from '@angular/core';
import { AbstractControl, FormControl, FormGroup, ValidationErrors } from '@angular/forms';
import { greet, LChart, ExampleKind } from 'l-chart';
import { Subject } from 'rxjs';
import { filter, takeUntil } from 'rxjs/operators';
import { ChartType } from './enum/chart-type';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.less']
})
export class AppComponent implements AfterViewInit, OnDestroy, OnInit {

  @ViewChild('canvas')
  canvas!: ElementRef<HTMLCanvasElement>;

  private gl: WebGLRenderingContext | undefined;

  types = ChartType;

  private chart: LChart | undefined;
  private destroy$ = new Subject<void>();

  form = new FormGroup(
    {
      'size': new FormControl(10),
      'type': new FormControl(ChartType.sin),
      'fromX': new FormControl(0),
      'toX': new FormControl(2 * Math.PI),
      'fromY': new FormControl(-1),
      'toY': new FormControl(1),
    },
    this.checkMode.bind(this)
  );

  public ngOnInit(): void {
    this.form.controls.fromX.disable({ emitEvent: false });
    this.form.controls.fromY.disable({ emitEvent: false });
    this.form.controls.toX.disable({ emitEvent: false });
    this.form.controls.toY.disable({ emitEvent: false });

    this.form.valueChanges.pipe(
      takeUntil(this.destroy$),
      filter(_ => !this.form.errors)
    )
      .subscribe((e: IViewModel) => this.applyModel(e));
    this.form.controls.type.valueChanges.subscribe((type: ChartType) => {
      switch (type) {
        case ChartType.sin:
          {
            const ctrls = this.form.controls;
            ctrls.fromX.disable({ emitEvent: false });
            ctrls.fromY.disable({ emitEvent: false });
            ctrls.toX.disable({ emitEvent: false });
            ctrls.toY.disable({ emitEvent: false });
            this.form.setValue(
              {
                type: ChartType.sin,
                fromX: 0,
                toX: 2 * Math.PI,
                fromY: -1,
                toY: 1,
                size: ctrls.size.value
              },
              { emitEvent: false }
            );
          }
          break;
        case ChartType.custom:
          {
            const ctrls = this.form.controls;
            ctrls.fromX.enable({ emitEvent: false });
            ctrls.fromY.enable({ emitEvent: false });
            ctrls.toX.enable({ emitEvent: false });
            ctrls.toY.enable({ emitEvent: false });
            this.form.setValue(
              {
                type: ChartType.custom,
                fromX: -10000,
                fromY: -10000,
                toX: 10000,
                toY: 10000,
                size: ctrls.size.value
              },
              { emitEvent: false }
            );
          }
          break;
      }
    });
  }

  private applyModel(source: IViewModel): void {
    if (this.chart) {
      if (source.type === ChartType.custom) {
        this.chart.test(ExampleKind.Custom, source.size, source.fromX!, source.toX!, source.fromY!, source.toY!);
      } else {
        this.chart.test(ExampleKind.Sin, source.size, 0, 2 * Math.PI, -1, 1);
      }
    }
  }

  private checkMode(control: AbstractControl): ValidationErrors | null {
    const value: IViewModel = control.value;
    let result: ValidationErrors | null = value.size < 2
      ? { size: true }
      : null;
    if (value.type === ChartType.custom) {
      const model = this.normalizeViewModel(value);
      if (model.fromX >= model.toX || model.fromY >= model.toY) {
        const mark = { rangeX: true };
        result = result ? { ...result, ...mark } : mark;
      }
      if (model.fromY >= model.toY) {
        const mark = { rangeY: true };
        result = result ? { ...result, ...mark } : mark;
      }
    }
    return result;
  }

  private normalizeViewModel(source: IViewModel): Required<IViewModel> {
    return {
      size: source.size,
      type: source.type,
      fromX: source.fromX!,
      fromY: source.fromY!,
      toX: source.toX!,
      toY: source.toY!,
    }
  }

  public ngAfterViewInit(): void {
    this.canvas.nativeElement.width = this.canvas.nativeElement.clientWidth;
    this.canvas.nativeElement.height = this.canvas.nativeElement.clientHeight;

    this.gl = this.canvas.nativeElement.getContext("webgl")!;
    this.chart = LChart.new(this.gl!);
    this.applyModel(this.form.value);
  }

  public ngOnDestroy(): void {
    if (this.chart) {
      this.chart.free();
      this.chart = undefined;
    }
    this.destroy$.next();
    this.destroy$.complete();
  }
}


interface IViewModel {
  type: ChartType,
  size: number,
  fromX?: number,
  fromY?: number,
  toX?: number,
  toY?: number,
}
