from gradio_client import Client
import json

try:
    print("🔍 Conectando ao Omni-Video-Factory para mapeamento...")
    client = Client("FrameAI4687/Omni-Video-Factory")
    
    # O comando view_api() retorna uma estrutura com todos os detalhes
    # Vamos imprimir de forma organizada para você ler no log do GitHub
    print("\n--- MAPA DA API ENCONTRADO ---")
    api_info = client.view_api(return_format="dict")
    print(json.dumps(api_info, indent=2))
    print("\n--- FIM DO MAPA ---")

except Exception as e:
    print(f"❌ Erro ao inspecionar: {e}")
