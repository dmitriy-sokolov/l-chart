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
      .subscribe((e: IViewModel) => {
        if (this.chart) {
          if (e.type === ChartType.custom) {
            this.chart.test(this.gl!, ExampleKind.Custom, e.size, e.fromX!, e.toX!, e.fromY!, e.toY!);
          } else {
            this.chart.test(this.gl!, ExampleKind.Sin, e.size, 0, 2 * Math.PI, -1, 1);
          }
        }
        console.log('value is', e);
      });
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
    // if (gl) {
    //   // установка размеров области рисования
    //   // Set the WebGL context to be the full size of the canvas
    //   gl.viewport(0, 0, this.canvas.nativeElement.width, this.canvas.nativeElement.height);

    //   console.log(this.canvas.nativeElement.width, this.canvas.nativeElement.height);
    //   // установка шейдеров 
    //   const idx = this.initShaders(gl);

    //   // установка буфера вершин
    //   this.initBuffers(gl);
    //   // покрасим в красный цвет фон
    //   gl.clearColor(1.0, 0.0, 0.0, 1.0);
    //   // отрисовка сцены
    //   this.draw(gl, idx);
    // }

    this.chart = LChart.new();
    this.chart.draw(this.gl);
  }

  public ngOnDestroy(): void {
    if (this.chart) {
      this.chart.free();
      this.chart = undefined;
    }
    this.destroy$.next();
    this.destroy$.complete();
  }

  private initShaders(
    gl: WebGLRenderingContext
  ): number {
    const vertexShader = gl.createShader(gl.VERTEX_SHADER)!;
    gl.shaderSource(vertexShader,
      `attribute vec3 aVertexPosition;
        void main(void) {
          gl_Position = vec4(aVertexPosition, 1.0);
        }`);
    // компилируем шейдер
    gl.compileShader(vertexShader);

    const fragmentShader = gl.createShader(gl.FRAGMENT_SHADER)!;
    gl.shaderSource(fragmentShader,
      `void main(void) {
          gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
        }`);
    // компилируем шейдер
    gl.compileShader(fragmentShader);

    //создаем объект программы шейдеров
    const shaderProgram = gl.createProgram()!;

    // прикрепляем к ней шейдеры
    gl.attachShader(shaderProgram, vertexShader);
    gl.attachShader(shaderProgram, fragmentShader);

    // связываем программу с контекстом webgl
    gl.linkProgram(shaderProgram);

    gl.useProgram(shaderProgram);
    // установка атрибута программы
    const vertexPositionAttribute = gl.getAttribLocation(shaderProgram, "aVertexPosition");
    // подключаем атрибут для использования
    gl.enableVertexAttribArray(vertexPositionAttribute);
    return vertexPositionAttribute;
  }

  // установка буфера вершин 
  private initBuffers(
    gl: WebGLRenderingContext
  ): void {
    const vertices = [
      -0.5, -0.5, 0.0,
      -0.5, 0.5, 0.0,
      0.5, 0.5, 0.0,
      0.5, -0.5, 0.0
    ];

    const indices = [0, 1, 2, 0, 3, 2];

    // установка буфера вершин
    const vertexBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, vertexBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(vertices), gl.STATIC_DRAW);

    // создание буфера индексов
    const indexBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexBuffer);
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, new Uint16Array(indices), gl.STATIC_DRAW);
  }

  // отрисовка 
  private draw(
    gl: WebGLRenderingContext,
    idx: number
  ): void {
    // установка области отрисовки
    gl.viewport(0, 0, this.canvas.nativeElement.width, this.canvas.nativeElement.height);

    gl.clear(gl.COLOR_BUFFER_BIT);

    // указываем, что каждая вершина имеет по три координаты (x, y, z)
    gl.vertexAttribPointer(
      idx, // shaderProgram.vertexPositionAttribute,
      3, // vertexBuffer.itemSize,
      gl.FLOAT,
      false,
      0,
      0
    );
    // отрисовка примитивов - линий
    gl.drawElements(
      gl.LINES,
      6, // indexBuffer.numberOfItems,
      gl.UNSIGNED_SHORT,
      0
    );
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
