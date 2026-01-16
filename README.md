# 🌈 Iris - Mensageira dos Devs

<div align="center">
  <img src="assets/logo.png" alt="Iris Logo" width="128" height="128">
  
  **Conectando desenvolvedores às suas aplicações**
  
  *Iris é a deusa grega do arco-íris e mensageira dos deuses, servindo como ponte entre o Olimpo e os mortais, entregando comandos e mensagens com velocidade.*
</div>

---

Assim como a deusa Iris levava mensagens entre os deuses e os mortais, este aplicativo serve como **ponte entre você e suas aplicações**, executando comandos e conectando seus projetos de forma rápida e elegante.

Desenvolvido em **Rust** para desenvolvedores que trabalham com múltiplas tecnologias e precisam de um hub centralizado para gerenciar e lançar suas aplicações.

## ✨ Funcionalidades

- 🎯 **Adicionar Aplicações**: Configure suas aplicações com nome, ícone, diretório e comandos
- 🎨 **Ícones de Tecnologias**: Escolha entre centenas de ícones de linguagens e frameworks
- 📝 **Comandos Personalizáveis**: Adicione quantos comandos forem necessários
- 🔀 **Reordenação**: Reorganize a ordem dos comandos facilmente
- 🔍 **Busca**: Encontre rapidamente suas aplicações
- 💾 **Persistência**: Configurações salvas automaticamente
- 💻 **Terminal Nativo**: Abre um novo terminal Windows e executa os comandos em sequência
- 🎮 **Controle de Processos**: 
  - ▶ **Executar**: Inicia a aplicação em um novo terminal
  - ■ **Stop**: Para a aplicação em execução
  - ↻ **Restart**: Reinicia a aplicação
- 🟢 **Indicação Visual**: Cards ficam verdes quando a aplicação está rodando
- ⚡ **Suporte a Scripts Interativos**: Detecta automaticamente inputs para scripts `.bat`
- 📤 **Exportar/Importar**: Compartilhe suas configurações com outros devs

## 📸 Screenshot

```
┌────────────────────────────────────────────────────────────┐
│  🌈 Iris - Mensageira dos Devs              [➕ Nova Aplicação]  │
├────────────────────────────────────────────────────────────┤
│                                                                │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │ ⚛️ React App    │  │ 🐍 Python API  │  │ 🦀 Rust Server │  │
│  │ 📁 /projetos    │  │ 📁 /api        │  │ 📁 /server     │  │
│  │ ⚡ 2 comandos   │  │ ⚡ 3 comandos   │  │ ⚡ 2 comandos   │  │
│  │                 │  │                 │  │                 │  │
│  │ [▶ Executar]    │  │ [■ Stop][↻]    │  │ [▶ Executar]    │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
│        (borda verde = executando)                              │
│                                                                │
├────────────────────────────────────────────────────────────┤
│  📦 3 aplicações   ▶ 1 em execução         Feito com ❤ em Rust  │
└────────────────────────────────────────────────────────────┘
```

## 📦 Instalação

### Pré-requisitos
- [Rust](https://rustup.rs/) instalado

### Compilar e Executar

```bash
# Clone o repositório
git clone <repo-url>
cd iris

# Executar em modo desenvolvimento
cargo run

# Compilar versão release (otimizada)
cargo build --release
```

O executável será gerado em `target/release/iris.exe`

## 🚀 Como Usar

1. **Adicionar uma aplicação**: Clique em "➕ Nova Aplicação"
2. **Configure**:
   - 🎨 Escolha um ícone para identificar a aplicação (React, Python, Docker, etc.)
   - 📝 Defina o nome da aplicação
   - 📁 Selecione a pasta do projeto (Working Directory)
   - ⚡ Adicione os comandos na ordem desejada
3. **Executar**: Clique em "▶ Executar" no card da aplicação

### Exemplos de Configuração

**⚛️ Aplicação React:**
```
Ícone: react
Pasta: C:\projetos\minha-app-react
Comandos:
  1. npm install
  2. npm run dev
```

**🐍 API Python:**
```
Ícone: python
Pasta: C:\projetos\minha-api
Comandos:
  1. pip install -r requirements.txt
  2. python main.py
```

**🦀 Servidor Rust:**
```
Ícone: rust
Pasta: C:\projetos\meu-servidor
Comandos:
  1. cargo build --release
  2. cargo run
```

**🍃 Projeto Spring (com seleção de versão Node):**
```
Ícone: spring
Pasta: C:\projetos\spring-app
Comandos:
  1. setPath.bat      ← Script interativo
  2. 18.12.0          ← Input automático (versão do Node)
  3. mvn spring-boot:run
```

## 🗂️ Onde ficam as configurações?

As configurações são salvas em:
```
%APPDATA%\iris\config.json
```

## 📤 Exportar e Importar Configurações

Iris permite **compartilhar suas configurações** com outros desenvolvedores!

### Exportar
1. Clique no botão ⚙️ no canto superior direito
2. Selecione "📤 Exportar configurações"
3. Escolha onde salvar o arquivo `iris-config.json`

### Importar
1. Clique no botão ⚙️ no canto superior direito
2. Selecione "📥 Importar configurações"
3. Selecione o arquivo `iris-config.json` recebido
4. As aplicações serão **adicionadas** às suas existentes

> 💡 **Dica**: Você pode compartilhar o arquivo de configuração com seu time para padronizar os projetos!

## 🎨 Ícones Disponíveis

Iris vem com **centenas de ícones** de tecnologias, incluindo:

| Categoria | Exemplos |
|-----------|----------|
| **Frontend** | react, angular, vue, svelte, nextjs |
| **Backend** | nodejs, python, java, spring, dotnet |
| **Linguagens** | typescript, rust, go, kotlin, swift |
| **DevOps** | docker, kubernetes, aws, azure, jenkins |
| **Banco de Dados** | postgresql, mongodb, mysql, redis |
| **Mobile** | flutter, react, android, ios |

Digite o nome da tecnologia no filtro para encontrar o ícone!

## 🛠️ Tecnologias Utilizadas

- **[Rust](https://www.rust-lang.org/)** - Linguagem de programação
- **[egui/eframe](https://github.com/emilk/egui)** - Interface gráfica
- **[resvg](https://github.com/RazrFalcon/resvg)** - Renderização de SVG
- **[serde](https://serde.rs/)** - Serialização JSON
- **[rfd](https://github.com/PolyMeilex/rfd)** - Diálogos de arquivo nativos

## 🤝 Contribuindo

Contribuições são bem-vindas! Sinta-se à vontade para:

1. Fazer um Fork do projeto
2. Criar uma branch para sua feature (`git checkout -b feature/AmazingFeature`)
3. Commit suas mudanças (`git commit -m 'Add some AmazingFeature'`)
4. Push para a branch (`git push origin feature/AmazingFeature`)
5. Abrir um Pull Request

## 📝 Licença

Distribuído sob a licença MIT. Veja `LICENSE` para mais informações.

---

<div align="center">
  <sub>Feito com ❤️ em Rust por desenvolvedores, para desenvolvedores.</sub>
  <br>
  <sub>🌈 <i>"Como a deusa Iris conectava os deuses aos mortais, este app conecta você às suas aplicações."</i></sub>
</div>