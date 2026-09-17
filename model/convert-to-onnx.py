import sys
import torch

def convert(pt_path, onnx_path):
    # tch-rs が保存した TorchScript モデルを直接ロード
    model = torch.jit.load(pt_path, map_location="cpu")
    model.eval()

    # 入力テンソルの形状を実際の環境に合わせて設定
    # 例: [batch_size, channels, height, width] -> [1, 8, 16, 16]
    dummy_input = torch.randn(1, 8, 16, 16)

    torch.onnx.export(
        model,
        dummy_input,
        onnx_path,
        export_params=True,
        opset_version=14,
        do_constant_folding=True,
        input_names=["input"],
        output_names=["output"],
        dynamic_axes={"input": {0: "batch_size"}, "output": {0: "batch_size"}}
    )
    print(f"[Python] Converted TorchScript {pt_path} -> {onnx_path}")

if __name__ == "__main__":
    if len(sys.argv) > 2:
        convert(sys.argv[1], sys.argv[2])